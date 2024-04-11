use ahora_app::{App, AppInput};
use embassy_futures::select::{select3, Either3};

use crate::{
    battery::BATTERY_DATA, ble::APPLE_MEDIA_SERVICE_DATA, display::SpiDisplay,
    timer::INTERNAL_TIME_DATA,
};

pub async fn run(mut display: SpiDisplay) -> ! {
    let mut app = App::init(&mut display).unwrap();

    loop {
        let event = match select3(
            // A signal per message type is used here, rather than using a single
            // channel of the Event type, because if multiple events of the
            // same type end up in the channel we only want the latest.
            //
            // Although in the future we may have some event types that we
            // do want to queue, for example if there are multiple touch screen
            // actions we probably should handle all of them rather than just
            // the latest.
            APPLE_MEDIA_SERVICE_DATA.wait(),
            BATTERY_DATA.wait(),
            INTERNAL_TIME_DATA.wait(),
        )
        .await
        {
            Either3::First(e) => AppInput::AppleMedia(e),
            Either3::Second(e) => {
                // The battery task immediately signals this event on startup so we
                // don't need to draw the battery un-conditionally on startup.
                AppInput::Battery(e)
            }
            Either3::Third(current_time) => {
                // The timer task signals this event on periodically so we
                // don't need to draw the time un-conditionally on startup.
                AppInput::Time(current_time.into())
            }
        };
        app.handle_event(&mut display, event).unwrap();
    }
}
