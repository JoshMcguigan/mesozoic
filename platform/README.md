# Mesozoic - PineTime Platform

## Debugger connection

The one-time setup and firmware flashing require connecting to the debug pins of the PineTime as described in the [PineTime Devkit Wiring Guide](https://pine64.org/documentation/PineTime/Further_information/Devkit_wiring/).

## One-time setup

* Download [SoftDevice S132](https://www.nordicsemi.com/Products/Development-software/S132/Download?lang=en#infotabs)
  * Version 7.3.0 has been confirmed to work
* Extract the *.hex file from the downloaded zip
* Install probe-rs
  * `cargo install probe-rs --features cli`
* Erase the flash
  * `probe-rs erase --chip nrf52832_xxAA`
* Flash the SoftDevice
  * `probe-rs download --verify --format hex --chip nRF52832_xxAA softdevice.hex`

## Flashing firmware

Running the firmware while connected to the debugger: `cargo run`

Flashing the firmware to run disconnected from the debugger: `cargo flash --chip nRF52832_xxAA`

## Bluetooth connection

* Using some BLE explorer app (I use EFR Connect) connect to the device named PINE
* Read the battery level characteristic from the battery service
* Accept the pairing request
