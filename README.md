# Mesozoic - PineTime Firmware

Mesozoic is experimental firmware for the [PineTime Smartwatch](https://pine64.org/devices/pinetime/).

The project is made up of several crates:

* app - application layer code
  * compiled into both the embedded firmware and the simulator
* pinetime - low level firmware code supporting the PineTime hardware
* sim - Mesozoic / PineTime simulator

## Quick start (Simulator)

The fastest way to get started with Mesozoic is using the simulator.

```sh
# See .cargo/config for the full definition of this alias.
cargo msim
```

### Simulator dependencies

The simulator uses SDL2 and its development libraries. Installation instructions are available [here](https://github.com/embedded-graphics/simulator?tab=readme-ov-file#setup).

## Using Mesozoic on real hardware

### Setup

#### Debugger connection

The one-time setup and firmware flashing require connecting to the debug pins of the PineTime as described in the [PineTime Devkit Wiring Guide](https://pine64.org/documentation/PineTime/Further_information/Devkit_wiring/).

#### One-time setup

* Download [SoftDevice S132](https://www.nordicsemi.com/Products/Development-software/S132/Download?lang=en#infotabs)
  * Version 7.3.0 has been confirmed to work
* Extract the *.hex file from the downloaded zip
* Install probe-rs
  * `cargo install probe-rs --features cli`
* Erase the flash
  * `probe-rs erase --chip nrf52832_xxAA`
* Flash the SoftDevice
  * `probe-rs download --verify --format hex --chip nRF52832_xxAA softdevice.hex`

#### Bluetooth connection

* Using some BLE explorer app (I use EFR Connect) connect to the device named PINE
* Read the battery level characteristic from the battery service
* Accept the pairing request

### Flashing Mesozoic to the PineTime

```sh
# See .cargo/config for the full definition of this alias.
cargo mpinetime
```

#### Toolchain setup

You may need to add the appropriate rust toolchain:

```sh
rustup target add thumbv7em-none-eabihf
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.