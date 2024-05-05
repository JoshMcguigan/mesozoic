use std::{str::FromStr, time::Instant};

use mesozoic_app::{
    interface::{
        AppInput, AppOutput, AppleMediaServiceData, BatteryData, Gesture, MediaControl, TimeOfDay,
        Touch, TouchType, LCD_H, LCD_W,
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
    let mut window = Window::new("Mesozoic", &OutputSettings::default());
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

    let mut charging = true;
    let voltage = 4.1;
    app.handle_event(
        &mut display,
        start_time.elapsed().as_millis() as u64,
        AppInput::Battery(BatteryData { charging, voltage }),
    )
    .unwrap();

    let mut audio_index = 0;
    let audio = vec![
        AppleMediaServiceData {
            artist: ArrayString::from_str("Rustacean Station").unwrap(),
            album: ArrayString::from_str("April 28, 2023").unwrap(),
            title: ArrayString::from_str("Rust Embedded WG").unwrap(),
        },
        AppleMediaServiceData {
            artist: ArrayString::from_str("Chats with James").unwrap(),
            album: ArrayString::from_str("September 29, 2023").unwrap(),
            title: ArrayString::from_str("014 - Steve Klabnik").unwrap(),
        },
    ];

    'running: loop {
        window.update(&display);

        let mut events = window.events();
        let app_input = if let Some(event) = events.next() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                    Keycode::P => AppInput::AppleMedia(audio[audio_index].clone()),
                    Keycode::B => {
                        charging = !charging;
                        AppInput::Battery(BatteryData { charging, voltage })
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
                _ => AppInput::Tick,
            }
        } else {
            AppInput::Tick
        };

        match app
            .handle_event(
                &mut display,
                start_time.elapsed().as_millis() as u64,
                app_input,
            )
            .unwrap()
        {
            // TODO the nested calls to app.handle_event here are really messy
            Some(AppOutput::MediaControl(MediaControl::NextTrack)) => {
                audio_index += 1;
                audio_index %= audio.len();
                app.handle_event(
                    &mut display,
                    start_time.elapsed().as_millis() as u64,
                    AppInput::AppleMedia(audio[audio_index].clone()),
                )
                .unwrap();
            }
            Some(AppOutput::MediaControl(MediaControl::PreviousTrack)) => {
                if audio_index > 0 {
                    audio_index -= 1;
                } else {
                    audio_index = audio.len() - 1;
                }
                app.handle_event(
                    &mut display,
                    start_time.elapsed().as_millis() as u64,
                    AppInput::AppleMedia(audio[audio_index].clone()),
                )
                .unwrap();
            }
            Some(AppOutput::MediaControl(_)) => {
                // Sim doesn't do anything with this for now
            }
            None => { // do nothing
            }
        };
    }

    Ok(())
}
