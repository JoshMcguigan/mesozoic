use std::{str::FromStr, time::Instant};

use ahora_app::{App, AppInput, AppleMediaServiceData, BatteryData, TimeOfDay, LCD_H, LCD_W};

use arrayvec::ArrayString;
use embedded_graphics::geometry::Size;
use embedded_graphics_simulator::{
    sdl2::Keycode, OutputSettings, SimulatorDisplay, SimulatorEvent, Window,
};

type DisplayColor = embedded_graphics::pixelcolor::Rgb565;

fn main() -> Result<(), core::convert::Infallible> {
    let mut display: SimulatorDisplay<DisplayColor> =
        SimulatorDisplay::new(Size::new(LCD_W as u32, LCD_H as u32));
    let mut window = Window::new("Ahora", &OutputSettings::default());
    let start_time = Instant::now();

    let mut app = App::init(&mut display, start_time.elapsed().as_millis() as u64).unwrap();

    app.handle_event(
        &mut display,
        start_time.elapsed().as_millis() as u64,
        AppInput::Time(TimeOfDay {
            hours: 11,
            minutes: 22,
            seconds: 33,
            ..Default::default()
        }),
    )
    .unwrap();

    app.handle_event(
        &mut display,
        start_time.elapsed().as_millis() as u64,
        AppInput::AppleMedia(AppleMediaServiceData {
            artist: ArrayString::from_str("Gus Dapperton").unwrap(),
            album: ArrayString::from_str("Orca").unwrap(),
            title: ArrayString::from_str("Post Humorous").unwrap(),
        }),
    )
    .unwrap();

    app.handle_event(
        &mut display,
        start_time.elapsed().as_millis() as u64,
        AppInput::Battery(BatteryData { charging: true }),
    )
    .unwrap();

    'running: loop {
        window.update(&display);

        // TODO refactor this to also send a tick event at the rate requested by
        // the app
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyUp { keycode, .. } => {
                    match keycode {
                        Keycode::Up => app.handle_event(
                            &mut display,
                            start_time.elapsed().as_millis() as u64,
                            AppInput::Battery(BatteryData { charging: true }),
                        ),
                        Keycode::Down => app.handle_event(
                            &mut display,
                            start_time.elapsed().as_millis() as u64,
                            AppInput::Battery(BatteryData { charging: false }),
                        ),
                        _ => continue,
                    }
                    .unwrap();
                }
                SimulatorEvent::MouseButtonUp { point: _, .. } => {}
                _ => {}
            }
        }
    }

    Ok(())
}
