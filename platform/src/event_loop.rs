use ahora_app::{App, interface::AppInput};
use embassy_futures::select::{select4, Either4::*};
use embassy_time::Instant;

use crate::{
    battery::BATTERY_DATA,
    ble::{APPLE_MEDIA_SERVICE_DATA, TIME_SERVICE_DATA},
    display::SpiDisplay,
    tick::TICK,
};

pub async fn run(mut display: SpiDisplay) -> ! {
    let mut app = App::init(&mut display, Instant::now().as_millis()).unwrap();

    loop {
        let event = match select4(
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
            TIME_SERVICE_DATA.wait(),
            TICK.wait(),
        )
        .await
        {
            First(e) => AppInput::AppleMedia(e),
            Second(e) => AppInput::Battery(e),
            Third(current_time) => AppInput::Time(current_time.into()),
            Fourth(_) => AppInput::Tick,
        };
        app.handle_event(&mut display, Instant::now().as_millis(), event)
            .unwrap();
    }
}
