use embassy_futures::select::{select, Either};
use embassy_time::Timer;

use crate::ble::{CurrentTime, TIME_SERVICE_DATA};

pub static INTERNAL_TIME_DATA: embassy_sync::signal::Signal<
    embassy_sync::blocking_mutex::raw::ThreadModeRawMutex,
    CurrentTime,
> = embassy_sync::signal::Signal::new();

#[embassy_executor::task]
pub async fn task() {
    let mut time = CurrentTime::default();

    loop {
        INTERNAL_TIME_DATA.signal(time.clone());

        // We could implement a much more accurate way to move the clock forward, but
        // for now this will do.
        match select(TIME_SERVICE_DATA.wait(), Timer::after_secs(1)).await {
            Either::First(current_time) => {
                time = current_time;
            }
            Either::Second(_) => {
                if time.seconds >= 59 {
                    time.seconds = 0;

                    if time.minutes >= 59 {
                        time.minutes = 0;

                        if time.hours >= 23 {
                            time.hours = 0;
                        } else {
                            time.hours += 1;
                        }
                    } else {
                        time.minutes += 1;
                    }
                } else {
                    time.seconds += 1;
                }
            }
        }
    }
}
