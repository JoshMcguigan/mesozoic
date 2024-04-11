# Ahora - PineTime Firmware

This repo hold multiple crates. A cargo workspace can't be used because we need to compile some dependencies in more than one ahora crate, but using different features that aren't compatible.

* app - Ahora application layer code
* platform - Ahora low level firmware code
* sim - Ahora / PineTime simulator
