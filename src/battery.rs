use embassy_nrf::{gpio::{Input, Pull}, peripherals::{P0_12, P0_31}};

pub static BATTERY_DATA: embassy_sync::signal::Signal<
    embassy_sync::blocking_mutex::raw::ThreadModeRawMutex,
    BatteryData,
> = embassy_sync::signal::Signal::new();

pub struct BatteryData {
    pub charging: bool,
}

#[embassy_executor::task]
pub async fn task(charging_indication_pin: P0_12, _battery_voltage_pin: P0_31) {
    let mut charging_indication_input = Input::new(charging_indication_pin, Pull::None);

    loop {
        charging_indication_input.wait_for_any_edge().await;

        BATTERY_DATA.signal(BatteryData { charging: charging_indication_input.is_high() });
    }
}
