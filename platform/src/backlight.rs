use embassy_nrf::{
    gpio::{Level, Output, OutputDrive},
    peripherals::{P0_14, P0_22, P0_23},
};

#[embassy_executor::task]
pub async fn task(backlight_low_pin: P0_14, _backlight_mid_pin: P0_22, _backlight_high_pin: P0_23) {
    // Turn on the backlight, then `forget` this pin to skip the drop implementation
    // which resets the configuration.
    core::mem::forget(Output::new(
        backlight_low_pin,
        // These pins are active low, so we are turning
        // the backlight ON here.
        Level::Low,
        OutputDrive::Standard,
    ));
}
