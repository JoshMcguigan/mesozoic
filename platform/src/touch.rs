use ahora_app::interface::Touch;
use defmt::{info, unwrap};
use embassy_nrf::{
    bind_interrupts,
    gpio::{Input, Pull},
    peripherals::{P0_06, P0_07, P0_10, P0_28, TWISPI0},
    twim::{self, Twim},
};

use crate::event_loop::TOUCH_DATA;

const TOUCH_CONTROLLER_ADDR: u8 = 0x15;

bind_interrupts!(struct Irqs {
    SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => twim::InterruptHandler<TWISPI0>;
});

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
        let starting_addr = 0;
        let mut buf = [0u8; 7];
        unwrap!(
            twi.write_read(TOUCH_CONTROLLER_ADDR, &[starting_addr], &mut buf)
                .await
        );

        let touch_event = Touch {
            gesture: unwrap!(buf[1].try_into()),
            event_type: unwrap!((buf[3] >> 6).try_into()),
            x: buf[4],
            y: buf[5],
        };

        TOUCH_DATA.send(touch_event).await;
    }
}
