use embassy_time::{Duration, Timer};

pub static TICK: embassy_sync::signal::Signal<
    embassy_sync::blocking_mutex::raw::ThreadModeRawMutex,
    (),
> = embassy_sync::signal::Signal::new();

/// This value is turned up to allow watching maximum fps.
///
/// Something like 10hz is probably more appropriate.
const DISPLAY_ON_TICK_DURATION: Duration = Duration::from_hz(60);

#[embassy_executor::task]
pub async fn task() {
    loop {
        Timer::after(DISPLAY_ON_TICK_DURATION).await;
        TICK.signal(());
    }
}
