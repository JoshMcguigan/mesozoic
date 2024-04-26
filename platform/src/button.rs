use embassy_nrf::{
    gpio::{Input, Level, Output, OutputDrive, Pull},
    peripherals::{P0_13, P0_15},
};

use crate::event_loop::BUTTON_DATA;

#[embassy_executor::task]
pub async fn task(button_in_pin: P0_13, button_out_pin: P0_15) {
    // The output pin must be driven high to read the input.
    let _output = Output::new(button_out_pin, Level::High, OutputDrive::Standard);

    let mut button = Input::new(button_in_pin, Pull::None);
    loop {
        button.wait_for_rising_edge().await;
        BUTTON_DATA.send(()).await;
    }
}
