use core::borrow::Borrow;

use embedded_graphics::draw_target::DrawTarget;

use crate::{
    display::{draw_audio, draw_battery, draw_bg, draw_fps, draw_fullscreen_text, draw_time},
    interface::{
        AppInput, AppOutput, AppleMediaServiceData, BatteryData, DisplayColor, Gesture,
        MediaControl, TimeOfDay,
    },
};

pub struct App {
    active_window: ActiveWindow,
    time: TimeState,
    media: Option<AppleMediaServiceData>,
    battery: BatteryData,
    critical_error: Option<&'static str>,
}

#[derive(Clone, Copy)]
pub(crate) enum ActiveWindow {
    Main,
    Debug,
}

impl ActiveWindow {
    fn next(&self) -> Self {
        match self {
            ActiveWindow::Main => ActiveWindow::Debug,
            ActiveWindow::Debug => ActiveWindow::Main,
        }
    }
}

struct TimeState {
    ms_since_boot_when_time_last_specified: u64,
    last_specified_time: TimeOfDay,
    current_ms_since_boot: u64,
    /// Used only to tell how quickly we are processing updates.
    previous_ms_since_boot: u64,
}

impl TimeState {
    /// Calculates current time by looking at last specified time and adding
    /// the elapsed ms_since_boot.
    fn current_time(&self) -> TimeOfDay {
        // TODO handle rollover in ms_since_boot
        let ms_delta = self.current_ms_since_boot - self.ms_since_boot_when_time_last_specified;

        let mut hours_delta = ms_delta / 1000 / 60 / 60 % 24;
        let mut minutes_delta = ms_delta / 1000 / 60 % 60;
        let seconds_delta = ms_delta / 1000 % 60;

        let seconds = match self.last_specified_time.seconds + seconds_delta as u8 {
            seconds @ 0..=59 => seconds,
            seconds_plus_extra => {
                minutes_delta += 1;

                seconds_plus_extra % 60
            }
        };

        let minutes = match self.last_specified_time.minutes + minutes_delta as u8 {
            minutes @ 0..=59 => minutes,
            minutes_plus_extra => {
                hours_delta += 1;

                minutes_plus_extra % 60
            }
        };

        let hours = (self.last_specified_time.hours + hours_delta as u8) % 24;

        TimeOfDay {
            hours,
            minutes,
            seconds,
        }
    }
}

impl App {
    pub fn init<D, E>(display: &mut D, ms_since_boot: u64) -> Result<Self, D::Error>
    where
        D: DrawTarget<Color = DisplayColor, Error = E>,
        E: core::fmt::Debug,
    {
        let active_window = ActiveWindow::Main;

        let s = Self {
            active_window,
            time: TimeState {
                ms_since_boot_when_time_last_specified: ms_since_boot,
                last_specified_time: TimeOfDay::default(),
                current_ms_since_boot: ms_since_boot,
                previous_ms_since_boot: ms_since_boot,
            },
            media: None,
            // Placeholder battery data - this will be updated within 1 second by
            // the battery input polling.
            battery: BatteryData {
                charging: false,
                voltage: 3.5,
            },
            critical_error: None,
        };

        // Initialize by drawing the background once - this is a minor
        // optimization. Should be removed when we have a more holistic
        // display optimization solution.
        draw_bg(display)?;

        s.draw(display)?;

        Ok(s)
    }

    // TODO eventually this will return actions for the platform code to take, for
    // example turn on backlight
    //
    // TODO why is display special, compared to other "outputs" - it is hard to
    // communicate what we want to do to the display, perhaps we could with function
    // pointers? otherwise should the whole "device" get passed into these functions?
    pub fn handle_event<D, E>(
        &mut self,
        display: &mut D,
        ms_since_boot: u64,
        event: AppInput,
    ) -> Result<Option<AppOutput>, D::Error>
    where
        D: DrawTarget<Color = DisplayColor, Error = E>,
        E: core::fmt::Debug,
    {
        self.time.previous_ms_since_boot = self.time.current_ms_since_boot;
        self.time.current_ms_since_boot = ms_since_boot;

        let output = match event {
            AppInput::AppleMedia(e) => {
                self.media = Some(e);
                None
            }
            AppInput::Battery(e) => {
                self.battery = e;
                None
            }
            AppInput::Time(e) => {
                self.time.last_specified_time = e;
                self.time.ms_since_boot_when_time_last_specified = ms_since_boot;
                None
            }
            AppInput::Touch(touch) => {
                // We should only do this after pairing, and when the
                // touch overlaps with a play/pause button. For now
                // we check if we have media data, as an indication
                // we might be paired.
                if self.media.is_some() {
                    match touch.gesture {
                        Gesture::SingleClick => {
                            Some(AppOutput::MediaControl(MediaControl::TogglePlayPause))
                        }
                        Gesture::SlideRight => {
                            Some(AppOutput::MediaControl(MediaControl::NextTrack))
                        }
                        Gesture::SlideLeft => {
                            Some(AppOutput::MediaControl(MediaControl::PreviousTrack))
                        }
                        Gesture::SlideUp => Some(AppOutput::MediaControl(MediaControl::VolumeUp)),
                        Gesture::SlideDown => {
                            Some(AppOutput::MediaControl(MediaControl::VolumeDown))
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            AppInput::ButtonPressed => {
                let new_window = self.active_window.next();
                // TODO create switch_window method to always clear bg
                draw_bg(display)?;
                self.active_window = new_window;
                None
            }
            AppInput::Tick => {
                // this just triggers a re-draw
                None
            }
            AppInput::CriticalError(critial_error) => {
                self.critical_error = Some(critial_error);
                // TODO create switch_window method to always clear bg
                draw_bg(display)?;
                self.active_window = ActiveWindow::Debug;
                None
            }
        };

        self.draw(display)?;

        Ok(output)
    }

    fn draw<D, E>(&self, display: &mut D) -> Result<(), E>
    where
        D: DrawTarget<Color = DisplayColor, Error = E>,
        E: core::fmt::Debug,
    {
        match self.active_window {
            ActiveWindow::Main => {
                draw_battery(display, &self.battery)?;
                draw_time(display, self.time.current_time())?;
                if let Some(media_data) = self.media.borrow() {
                    draw_audio(display, &media_data.artist, &media_data.title)?;
                }
            }
            ActiveWindow::Debug => {
                if let Some(error_message) = self.critical_error {
                    draw_fullscreen_text(display, error_message)?;
                }
            }
        }

        // For now FPS is drawn at the bottom of every window.
        // TODO handle roll-over
        // max(1) to avoid divide by zero
        let fps =
            1000 / (self.time.current_ms_since_boot - self.time.previous_ms_since_boot).max(1);
        draw_fps(display, fps as u32)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use core::str::FromStr;

    use arrayvec::ArrayString;
    use embedded_graphics::geometry::Size;

    use crate::{
        interface::{BatteryData, LCD_H, LCD_W},
        test_infra::{assert_snapshot, function_name, SimDisplay},
    };

    use super::*;

    #[test]
    fn init_and_paired() {
        let test_name = function_name!();
        assert_eq!(
            "mesozoic_app::app::tests::init_and_paired", test_name,
            "this file is referenced from README, so the name cannot change"
        );
        let mut display = SimDisplay::new(Size::new(LCD_W as u32, LCD_H as u32));

        let ms_since_boot = 0;
        let mut app = App::init(&mut display, ms_since_boot).unwrap();
        app.handle_event(
            &mut display,
            ms_since_boot,
            AppInput::Battery(BatteryData {
                charging: true,
                voltage: 4.1,
            }),
        )
        .unwrap();
        app.handle_event(
            &mut display,
            ms_since_boot,
            AppInput::Time(TimeOfDay {
                hours: 10,
                minutes: 15,
                seconds: 01,
                ..Default::default()
            }),
        )
        .unwrap();
        app.handle_event(
            &mut display,
            ms_since_boot,
            AppInput::AppleMedia(AppleMediaServiceData {
                artist: ArrayString::from_str("Rustacean Station").unwrap(),
                album: ArrayString::from_str("April 28, 2023").unwrap(),
                title: ArrayString::from_str("Rust Embedded WG").unwrap(),
            }),
        )
        .unwrap();
        app.handle_event(
            &mut display,
            // Using this event to set FPS
            ms_since_boot + 17,
            AppInput::Tick,
        )
        .unwrap();

        assert_snapshot(test_name, display);
    }
}
