use std::{str::FromStr, time::Instant};

use ahora_app::{
    interface::{
        AppInput, AppleMediaServiceData, BatteryData, Gesture, TimeOfDay, Touch, TouchType, LCD_H,
        LCD_W,
    },
    App,
};

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
            hours: 23,
            minutes: 59,
            seconds: 58,
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

    let mut charging = true;
    app.handle_event(
        &mut display,
        start_time.elapsed().as_millis() as u64,
        AppInput::Battery(BatteryData { charging }),
    )
    .unwrap();

    'running: loop {
        window.update(&display);

        let mut events = window.events();
        let app_input = if let Some(event) = events.next() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::B => {
                        charging = !charging;
                        AppInput::Battery(BatteryData { charging })
                    }
                    Keycode::LShift => AppInput::ButtonPressed,
                    _ => continue,
                },
                SimulatorEvent::MouseButtonDown { point, .. } => AppInput::Touch(Touch {
                    gesture: Gesture::SingleClick,
                    event_type: TouchType::Down,
                    x: point.x as u8,
                    y: point.y as u8,
                }),
                _ => {
                    continue;
                }
            }
        } else {
            AppInput::Tick
        };

        app.handle_event(
            &mut display,
            start_time.elapsed().as_millis() as u64,
            app_input,
        )
        .unwrap();
    }

    Ok(())
}
