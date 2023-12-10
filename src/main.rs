#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _; // global logger
use embassy_nrf as _; // time driver
use panic_probe as _;

use defmt::unwrap;
use embassy_executor::Spawner;

mod ble;
mod display;
mod nrf;
mod touch;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = nrf::init();

    unwrap!(spawner.spawn(ble::task(ble::init(&spawner).await)));
    unwrap!(spawner.spawn(display::task(
        p.P0_14, p.P0_18, p.P0_25, p.TWISPI1, p.P0_02, p.P0_04, p.P0_03
    )));
    unwrap!(spawner.spawn(touch::task(p.P0_10, p.P0_28, p.TWISPI0, p.P0_06, p.P0_07)));
}
