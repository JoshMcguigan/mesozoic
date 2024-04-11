use std::str::FromStr;

use ahora_app::{App, AppleMediaServiceData, BatteryData, CurrentTime, AppInput, LCD_H, LCD_W};

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

    let mut app = App::init(&mut display).unwrap();

    app.handle_event(
        &mut display,
        AppInput::Time(CurrentTime {
            hours: 11,
            minutes: 22,
            seconds: 33,
            ..Default::default()
        }),
    )
    .unwrap();

    app.handle_event(
        &mut display,
        AppInput::AppleMedia(AppleMediaServiceData {
            artist: ArrayString::from_str("Gus Dapperton").unwrap(),
            album: ArrayString::from_str("Orca").unwrap(),
            title: ArrayString::from_str("Post Humorous").unwrap(),
        }),
    )
    .unwrap();

    app.handle_event(&mut display, AppInput::Battery(BatteryData { charging: true }))
        .unwrap();

    'running: loop {
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyUp { keycode, .. } => {
                    match keycode {
                        Keycode::Up => app.handle_event(
                            &mut display,
                            AppInput::Battery(BatteryData { charging: true }),
                        ),
                        Keycode::Down => app.handle_event(
                            &mut display,
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
