use embassy_nrf::peripherals::{P0_13, P0_15};

#[embassy_executor::task]
pub async fn task(_button_in_pin: P0_13, _button_out_pin: P0_15) {
    // Placeholder task for button driver
}
