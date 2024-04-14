use embassy_time::{Duration, Timer};

pub static TICK: embassy_sync::signal::Signal<
    embassy_sync::blocking_mutex::raw::ThreadModeRawMutex,
    (),
> = embassy_sync::signal::Signal::new();

const DISPLAY_ON_TICK_DURATION: Duration = Duration::from_hz(30);

#[embassy_executor::task]
pub async fn task() {
    loop {
        Timer::after(DISPLAY_ON_TICK_DURATION).await;
        TICK.signal(());
    }
}
