use core::{
    cell::{Cell, RefCell},
    mem,
};

use arrayvec::ArrayVec;
use defmt::{debug, info, unwrap};
use embassy_executor::{SendSpawner, Spawner};
use embassy_futures::select::{select, Either};
use mesozoic_app::interface::{AppleMediaServiceData, AppleMediaServiceString, MediaControl};
use nrf_softdevice::ble::gatt_server::builder::ServiceBuilder;
use nrf_softdevice::ble::gatt_server::characteristic::{Attribute, Metadata, Properties};
use nrf_softdevice::ble::gatt_server::{set_sys_attrs, RegisterError, WriteOp};
use nrf_softdevice::ble::security::{IoCapabilities, SecurityHandler};
use nrf_softdevice::ble::{
    gatt_client, gatt_server, peripheral, Connection, EncryptionInfo, GattValue, IdentityKey,
    MasterId, SecurityMode, Uuid,
};
use nrf_softdevice::{raw, Softdevice};
use static_cell::StaticCell;

use crate::event_loop::MEDIA_CONTROL;

pub static APPLE_MEDIA_SERVICE_DATA: embassy_sync::signal::Signal<
    embassy_sync::blocking_mutex::raw::ThreadModeRawMutex,
    AppleMediaServiceData,
> = embassy_sync::signal::Signal::new();

pub static TIME_SERVICE_DATA: embassy_sync::signal::Signal<
    embassy_sync::blocking_mutex::raw::ThreadModeRawMutex,
    CurrentTime,
> = embassy_sync::signal::Signal::new();

pub struct TaskParams {
    sd: &'static Softdevice,
    server: Server,
    spawner: SendSpawner,
}

const BATTERY_SERVICE: Uuid = Uuid::new_16(0x180f);
const BATTERY_LEVEL: Uuid = Uuid::new_16(0x2a19);

#[embassy_executor::task]
pub async fn softdevice_task(sd: &'static Softdevice) {
    sd.run().await;
}

#[derive(Debug, Clone, Copy)]
struct Peer {
    master_id: MasterId,
    key: EncryptionInfo,
    peer_id: IdentityKey,
}

pub struct Bonder {
    peer: Cell<Option<Peer>>,
    sys_attrs: RefCell<ArrayVec<u8, 62>>,
}

impl Default for Bonder {
    fn default() -> Self {
        Bonder {
            peer: Cell::new(None),
            sys_attrs: Default::default(),
        }
    }
}

impl SecurityHandler for Bonder {
    fn io_capabilities(&self) -> IoCapabilities {
        IoCapabilities::None
    }

    fn can_bond(&self, _conn: &Connection) -> bool {
        // default was true here, am I breaking something?
        false
    }

    fn display_passkey(&self, passkey: &[u8; 6]) {
        info!("The passkey is \"{:a}\"", passkey)
    }

    fn on_bonded(
        &self,
        _conn: &Connection,
        master_id: MasterId,
        key: EncryptionInfo,
        peer_id: IdentityKey,
    ) {
        debug!("storing bond for: id: {}, key: {}", master_id, key);

        // In a real application you would want to signal another task to permanently store the keys in non-volatile memory here.
        self.sys_attrs.borrow_mut().clear();
        self.peer.set(Some(Peer {
            master_id,
            key,
            peer_id,
        }));
    }

    fn get_key(&self, _conn: &Connection, master_id: MasterId) -> Option<EncryptionInfo> {
        debug!("getting bond for: id: {}", master_id);

        self.peer
            .get()
            .and_then(|peer| (master_id == peer.master_id).then_some(peer.key))
    }

    fn save_sys_attrs(&self, conn: &Connection) {
        debug!("saving system attributes for: {}", conn.peer_address());

        if let Some(peer) = self.peer.get() {
            if peer.peer_id.is_match(conn.peer_address()) {
                let mut sys_attrs = self.sys_attrs.borrow_mut();
                let capacity = sys_attrs.capacity();
                unsafe {
                    sys_attrs.set_len(capacity);
                }
                let len = unwrap!(gatt_server::get_sys_attrs(conn, &mut sys_attrs)) as u16;
                sys_attrs.truncate(usize::from(len));
                // In a real application you would want to signal another task to permanently store sys_attrs for this connection's peer
            }
        }
    }

    fn load_sys_attrs(&self, conn: &Connection) {
        let addr = conn.peer_address();
        debug!("loading system attributes for: {}", addr);

        let attrs = self.sys_attrs.borrow();
        // In a real application you would search all stored peers to find a match
        let attrs = if self
            .peer
            .get()
            .map(|peer| peer.peer_id.is_match(addr))
            .unwrap_or(false)
        {
            (!attrs.is_empty()).then_some(attrs.as_slice())
        } else {
            None
        };

        unwrap!(set_sys_attrs(conn, attrs));
    }
}

pub struct BatteryService {
    cccd_handle: u16,
}

impl BatteryService {
    pub fn new(sd: &mut Softdevice) -> Result<Self, RegisterError> {
        let mut service_builder = ServiceBuilder::new(sd, BATTERY_SERVICE)?;

        let attr = Attribute::new(&[0u8]).security(SecurityMode::JustWorks);
        let metadata = Metadata::new(Properties::new().read().notify());
        let characteristic_builder =
            service_builder.add_characteristic(BATTERY_LEVEL, attr, metadata)?;
        let characteristic_handles = characteristic_builder.build();

        let _service_handle = service_builder.build();

        Ok(BatteryService {
            cccd_handle: characteristic_handles.cccd_handle,
        })
    }

    pub fn on_write(&self, handle: u16, data: &[u8]) {
        if handle == self.cccd_handle && !data.is_empty() {
            info!("battery notifications: {}", (data[0] & 0x01) != 0);
        }
    }
}

pub struct Server {
    bas: BatteryService,
}

impl Server {
    pub fn new(sd: &mut Softdevice) -> Result<Self, RegisterError> {
        let bas = BatteryService::new(sd)?;

        Ok(Self { bas })
    }
}

impl gatt_server::Server for Server {
    type Event = ();

    fn on_write(
        &self,
        _conn: &Connection,
        handle: u16,
        _op: WriteOp,
        _offset: usize,
        data: &[u8],
    ) -> Option<Self::Event> {
        self.bas.on_write(handle, data);
        None
    }
}

const ATT_PAYLOAD_MAX_LEN: usize = 512;

// TODO replace this with ArrayVec?
struct MyVec {
    data: [u8; ATT_PAYLOAD_MAX_LEN],
    len: usize,
}

impl GattValue for MyVec {
    const MIN_SIZE: usize = 1;

    const MAX_SIZE: usize = ATT_PAYLOAD_MAX_LEN;

    fn from_gatt(data: &[u8]) -> Self {
        let mut saved_data = [0; ATT_PAYLOAD_MAX_LEN];
        saved_data[..data.len()].copy_from_slice(data);

        Self {
            data: saved_data,
            len: data.len(),
        }
    }

    fn to_gatt(&self) -> &[u8] {
        &self.data[..self.len]
    }
}

#[nrf_softdevice::gatt_client(uuid = "89D3502B-0F36-433A-8EF4-C502AD55F8DC")]
struct AppleMediaServiceClient {
    #[characteristic(uuid = "9B3C81D8-57B1-4A8A-B8DF-0E56F7CA51C2", write, notify)]
    remote_command: u8,
    #[characteristic(uuid = "2F7CABCE-808D-411F-9A0C-BB92BA96C102", write, notify)]
    entity_update: MyVec,
}

#[nrf_softdevice::gatt_client(uuid = "180f")]
struct BatteryServiceClient {
    #[characteristic(uuid = "2a19", read)]
    battery_level: u8,
}

#[nrf_softdevice::gatt_client(uuid = "1805")]
struct TimeServiceClient {
    #[characteristic(uuid = "2a2b", read)]
    current_time: CurrentTime,
}

#[derive(defmt::Format, Default, Clone)]
pub struct CurrentTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub day_of_week: u8,
    pub fractions_256: u8,
    pub adjust_reason: u8,
}

impl From<CurrentTime> for mesozoic_app::interface::TimeOfDay {
    fn from(value: CurrentTime) -> Self {
        Self {
            hours: value.hours,
            minutes: value.minutes,
            seconds: value.seconds,
        }
    }
}

impl GattValue for CurrentTime {
    const MIN_SIZE: usize = 10;

    const MAX_SIZE: usize = 10;

    fn from_gatt(data: &[u8]) -> Self {
        Self {
            // This unwrap is safe because we know statically that we've
            // passed in a slice of length 2.
            year: u16::from_le_bytes(data[0..2].try_into().unwrap()),
            month: data[2],
            day: data[3],
            hours: data[4],
            minutes: data[5],
            seconds: data[6],
            day_of_week: data[7],
            fractions_256: data[8],
            adjust_reason: data[9],
        }
    }

    fn to_gatt(&self) -> &[u8] {
        unimplemented!("we never write this so to_gatt is not needed")
    }
}

const ENTITY_ID_TRACK: u8 = 2;
const TRACK_ATTRIBUTE_ID_ARTIST: u8 = 0;
const TRACK_ATTRIBUTE_ID_ALBUM: u8 = 1;
const TRACK_ATTRIBUTE_ID_TITLE: u8 = 2;

#[embassy_executor::task]
pub async fn task_gatt_client(conn: Connection) {
    loop {
        let client: BatteryServiceClient = unwrap!(gatt_client::discover(&conn).await);
        let e = client.battery_level_read().await;
        info!("response {:?}", e);

        if e.is_ok() {
            break;
        }
    }

    let client: TimeServiceClient = unwrap!(gatt_client::discover(&conn).await);
    let e = client.current_time_read().await;
    info!("response {:?}", e);

    // iOS doesn't seem to send many notifications for the time service, so
    // for now we only send along the first value received rather than subscribe.
    TIME_SERVICE_DATA.signal(unwrap!(e));

    let client: AppleMediaServiceClient = unwrap!(gatt_client::discover(&conn).await);

    client.remote_command_cccd_write(true).await.unwrap();
    client.entity_update_cccd_write(true).await.unwrap();

    let mut a = [0; ATT_PAYLOAD_MAX_LEN];
    a[0] = ENTITY_ID_TRACK;
    a[1] = TRACK_ATTRIBUTE_ID_ARTIST;
    a[2] = TRACK_ATTRIBUTE_ID_ALBUM;
    a[3] = TRACK_ATTRIBUTE_ID_TITLE;
    let e = client.entity_update_write(&MyVec { data: a, len: 4 }).await;
    info!("entity_update write response {:?}", e);
    unwrap!(e);

    let mut artist = AppleMediaServiceString::new();
    let mut album = AppleMediaServiceString::new();
    let mut title = AppleMediaServiceString::new();

    // flush media control, because we don't want to act on commands received before pairing
    loop {
        if let Err(_) = MEDIA_CONTROL.try_receive() {
            break;
        }
    }

    // There is an issue here where iOS is either not sending, or we are missing, the initial
    // media information when we first connect. Then on further song changes, if only the title
    // changes, iOS only sends the changed data.

    loop {
        let notifications = gatt_client::run(&conn, &client, |event| match event {
            AppleMediaServiceClientEvent::EntityUpdateNotification(val) => {
                let entity_id = val.data[0];
                match entity_id {
                    2 => {
                        let attribute_id = val.data[1];
                        // These flags include a bool indicating the value was truncated
                        // but for now we ignore this
                        // let entity_update_flags = val.data[2];
                        let value = &val.data[3..val.len];

                        if let Ok(value_as_str) = core::str::from_utf8(value) {
                            let attribute = match attribute_id {
                                0 => {
                                    // TODO move this from up above to handle
                                    // the error as we do with fromutf8
                                    artist = AppleMediaServiceString::from(value_as_str).unwrap();
                                    "artist"
                                }
                                1 => {
                                    // TODO move this from up above to handle
                                    // the error as we do with fromutf8
                                    album = AppleMediaServiceString::from(value_as_str).unwrap();
                                    "album"
                                }
                                2 => {
                                    // TODO move this from up above to handle
                                    // the error as we do with fromutf8
                                    title = AppleMediaServiceString::from(value_as_str).unwrap();
                                    APPLE_MEDIA_SERVICE_DATA.signal(AppleMediaServiceData {
                                        artist,
                                        album,
                                        title,
                                    });
                                    // TODO may want to clear the data here so we don't
                                    // accidentally send stale data

                                    // TODO there is a bug here because iOS will only
                                    // send the values that change. So if we skip
                                    // from one song to another that has the exact
                                    // same title - our code would not generate
                                    // a data changed event.

                                    "title"
                                }
                                3 => "duration",
                                _ => "unknown",
                            };

                            info!("{}: {}", attribute, value_as_str);
                        } else {
                            info!("invalid utf8 received");
                        }
                    }
                    _ => info!("unknown entity ID"),
                };
            }
            AppleMediaServiceClientEvent::RemoteCommandNotification(val) => {
                info!("remote command notification: {}", val);
            }
        });
        match select(notifications, MEDIA_CONTROL.receive()).await {
            Either::First(_) => continue,
            Either::Second(command) => {
                unwrap!(
                    client
                        .remote_command_write(
                            &(match command {
                                MediaControl::TogglePlayPause => 2,
                            })
                        )
                        .await
                );
            }
        };
    }
}

pub async fn init(spawner: &Spawner) -> TaskParams {
    let config = nrf_softdevice::Config {
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_RC as u8,
            rc_ctiv: 16,
            rc_temp_ctiv: 2,
            accuracy: raw::NRF_CLOCK_LF_ACCURACY_500_PPM as u8,
        }),
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: 1,
            event_length: 24,
        }),
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 527 }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: 1408,
        }),
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: 1,
            periph_role_count: 1,
            central_role_count: 1,
            central_sec_count: 1,
            _bitfield_1: raw::ble_gap_cfg_role_count_t::new_bitfield_1(0),
        }),
        gap_device_name: Some(raw::ble_gap_cfg_device_name_t {
            p_value: b"PineTime" as *const u8 as _,
            current_len: 8,
            max_len: 8,
            write_perm: unsafe { mem::zeroed() },
            _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(
                raw::BLE_GATTS_VLOC_STACK as u8,
            ),
        }),
        ..Default::default()
    };

    let sd = Softdevice::enable(&config);
    let server = unwrap!(Server::new(sd));
    unwrap!(spawner.spawn(softdevice_task(sd)));

    TaskParams {
        sd,
        server,
        spawner: spawner.make_send(),
    }
}

#[embassy_executor::task]
pub async fn task(state: TaskParams) {
    let TaskParams {
        sd,
        server,
        spawner,
    } = state;

    #[rustfmt::skip]
    let adv_data = &[
        0x02, 0x01, raw::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
        0x03, 0x03, 0x0F, 0x18,
        0x05, 0x09, b'P', b'I', b'N', b'E',
    ];
    #[rustfmt::skip]
    let scan_data = &[
        0x11, // length
        0x15, // service soliciation
        // Apple media service UUID
        0xDC, 
        0xF8,
        0x55,
        0xAD,
        0x02,
        0xC5,
        0xF4,
        0x8E,
        0x3A,
        0x43,
        0x36,
        0x0F,
        0x2B,
        0x50,
        0xD3,
        0x89,
    ];

    static BONDER: StaticCell<Bonder> = StaticCell::new();
    let bonder = BONDER.init(Bonder::default());

    loop {
        let config = peripheral::Config::default();
        let adv = peripheral::ConnectableAdvertisement::ScannableUndirected {
            adv_data,
            scan_data,
        };
        let conn = unwrap!(peripheral::advertise_pairable(sd, adv, &config, bonder).await);

        unwrap!(spawner.spawn(task_gatt_client(conn.clone())));

        // Run the GATT server on the connection. This returns when the connection gets disconnected.
        let e = gatt_server::run(&conn.clone(), &server, |_| {
            // Do nothing
        })
        .await;
        info!("gatt_server run exited with error: {:?}", e);
    }
}
