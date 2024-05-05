use embassy_futures::select::select;
use embassy_nrf::{
    bind_interrupts,
    gpio::{Input, Pull},
    peripherals::{P0_12, P0_31, SAADC},
    saadc,
};
use embassy_time::Timer;
use mesozoic_app::interface::BatteryData;

pub static BATTERY_DATA: embassy_sync::signal::Signal<
    embassy_sync::blocking_mutex::raw::ThreadModeRawMutex,
    BatteryData,
> = embassy_sync::signal::Signal::new();

bind_interrupts!(struct Irqs {
    SAADC => embassy_nrf::saadc::InterruptHandler;
});

#[embassy_executor::task]
pub async fn task(
    charging_indication_pin: P0_12,
    saadc_periph: SAADC,
    mut battery_voltage_pin: P0_31,
) {
    let mut charging_indication_input = Input::new(charging_indication_pin, Pull::None);
    let channel_config = saadc::ChannelConfig::single_ended(&mut battery_voltage_pin);
    let mut battery_voltage_input =
        saadc::Saadc::new(saadc_periph, Irqs, Default::default(), [channel_config]);

    loop {
        // We want to share battery data immediately rather than waiting for the
        // first change, so we wait_for_any_edge only after the first time we signal
        // the data.

        let voltage = {
            let mut buf = [0; 1];
            battery_voltage_input.sample(&mut buf).await;

            2. * 3.3 * (buf[0] as f32) / 4095.
        };

        BATTERY_DATA.signal(BatteryData {
            // Charging indication is inverted, low means the battery is charging.
            charging: charging_indication_input.is_low(),
            voltage,
        });

        let _ = select(
            charging_indication_input.wait_for_any_edge(),
            Timer::after_secs(1),
        )
        .await;
    }
}
