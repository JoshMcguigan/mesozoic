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

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = nrf::init();

    unwrap!(spawner.spawn(display::task(p)));
    unwrap!(spawner.spawn(ble::task(ble::init(&spawner).await)));
}
