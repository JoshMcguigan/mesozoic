# Mesozoic - PineTime Firmware

This repo hold multiple crates. A cargo workspace can't be used because we need to compile some dependencies in more than one mesozoic crate, but using different features that aren't compatible.

* app - Mesozoic application layer code
* platform - Mesozoic low level firmware code
* sim - Mesozoic / PineTime simulator
