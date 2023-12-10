use defmt::{info, unwrap};
use embassy_nrf::{
    bind_interrupts,
    gpio::{Input, Pull},
    peripherals::{P0_06, P0_07, P0_10, P0_28, TWISPI0},
    twim::{self, Twim},
};

const TOUCH_CONTROLLER_ADDR: u8 = 0x15;

bind_interrupts!(struct Irqs {
    SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => twim::InterruptHandler<TWISPI0>;
});

#[derive(defmt::Format)]
struct TouchEvent {
    gesture: Gesture,
    event_type: EventType,
    x: u8,
    y: u8,
}

#[derive(defmt::Format)]
enum Gesture {
    None,
    SlideDown,
    SlideUp,
    SlideLeft,
    SlideRight,
    SingleClick,
    DoubleClick,
    LongPress,
}

#[derive(defmt::Format)]
enum EventType {
    Down,
    Up,
    Contact,
}

#[embassy_executor::task]
pub async fn task(
    _reset_pin: P0_10,
    interrupt_pin: P0_28,
    mut twi: TWISPI0,
    mut sda_pin: P0_06,
    mut scl_pin: P0_07,
) {
    let mut interrupt_pin = Input::new(interrupt_pin, Pull::None);

    loop {
        // TODO not sure which edge we should be watching for here, but probably just one not "any"
        interrupt_pin.wait_for_any_edge().await;
        info!("Touch controller awake, reading data..");

        // The twi peripheral is re-created for each read, so it can be dropped after we are
        // done. This saves power while we aren't actively communicating with the touch controller.
        let mut twi = Twim::new(
            &mut twi,
            Irqs,
            &mut sda_pin,
            &mut scl_pin,
            twim::Config::default(),
        );

        // TODO all unwraps here should be replaced with proper error handling
        let mut buf = [0u8; 7];
        unwrap!(twi.read(TOUCH_CONTROLLER_ADDR, &mut buf).await);

        let touch_event = TouchEvent {
            gesture: unwrap!(buf[1].try_into()),
            event_type: unwrap!((buf[3] >> 6).try_into()),
            x: buf[4],
            y: buf[5],
        };

        info!("detected touch event: {:?}", touch_event);
    }
}

impl TryFrom<u8> for Gesture {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Gesture::None),
            0x01 => Ok(Gesture::SlideDown),
            0x02 => Ok(Gesture::SlideUp),
            0x03 => Ok(Gesture::SlideLeft),
            0x04 => Ok(Gesture::SlideRight),
            0x05 => Ok(Gesture::SingleClick),
            0x0B => Ok(Gesture::DoubleClick),
            0x0C => Ok(Gesture::LongPress),
            other => Err(other),
        }
    }
}

impl TryFrom<u8> for EventType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EventType::Down),
            1 => Ok(EventType::Up),
            2 => Ok(EventType::Contact),
            other => Err(other),
        }
    }
}
