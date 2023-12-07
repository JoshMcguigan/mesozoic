pub fn init() -> embassy_nrf::Peripherals {
    let config = {
        let mut c = embassy_nrf::config::Config::default();
        c.gpiote_interrupt_priority = embassy_nrf::interrupt::Priority::P2;
        c.time_interrupt_priority = embassy_nrf::interrupt::Priority::P2;
        c
    };

    embassy_nrf::init(config)
}
