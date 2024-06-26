#![no_std]
#![no_main]

use defmt_rtt as _; // global logger
use embassy_nrf as _; // time driver
use panic_probe as _;

use defmt::unwrap;
use embassy_executor::Spawner;

mod event_loop;
mod nrf;
mod tasks;

use tasks::{backlight, battery, ble, button, display, haptics, tick, touch};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = nrf::init();

    unwrap!(spawner.spawn(backlight::task(p.P0_14, p.P0_22, p.P0_23)));
    unwrap!(spawner.spawn(battery::task(p.P0_12, p.SAADC, p.P0_31)));
    unwrap!(spawner.spawn(ble::task(ble::init(&spawner).await)));
    unwrap!(spawner.spawn(button::task(p.P0_13, p.P0_15)));
    unwrap!(spawner.spawn(haptics::task(p.P0_16)));
    unwrap!(spawner.spawn(tick::task()));
    unwrap!(spawner.spawn(touch::task(p.P0_10, p.P0_28, p.TWISPI0, p.P0_06, p.P0_07)));

    let display = display::create(p.P0_18, p.P0_25, p.TWISPI1, p.P0_02, p.P0_04, p.P0_03);
    event_loop::run(display).await;
}
