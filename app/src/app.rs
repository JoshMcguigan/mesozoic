use core::borrow::Borrow;

use arrayvec::ArrayString;
use embedded_graphics::draw_target::DrawTarget;

use crate::{
    display::{draw_audio, draw_battery, draw_bg, draw_fps, draw_time},
    interface::{AppInput, AppleMediaServiceData, DisplayColor, TimeOfDay},
};

pub struct App {
    active_window: ActiveWindow,
    time: TimeState,
    media: Option<AppleMediaServiceData>,
    charging: bool,
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
            charging: false,
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
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = DisplayColor, Error = E>,
        E: core::fmt::Debug,
    {
        self.time.previous_ms_since_boot = self.time.current_ms_since_boot;
        self.time.current_ms_since_boot = ms_since_boot;

        match event {
            AppInput::AppleMedia(e) => {
                self.media = Some(e);
            }
            AppInput::Battery(e) => {
                self.charging = e.charging;
            }
            AppInput::Time(e) => {
                self.time.last_specified_time = e;
                self.time.ms_since_boot_when_time_last_specified = ms_since_boot;
            }
            AppInput::Touch(touch) => {
                // TODO replace this with something reasonable

                use core::fmt::Write;
                let mut x = ArrayString::<512>::new();
                let mut y = ArrayString::<512>::new();
                write!(&mut x, "{}", touch.x).unwrap();
                write!(&mut y, "{}", touch.y).unwrap();
                self.media = Some(AppleMediaServiceData {
                    title: x,
                    artist: y,
                    album: ArrayString::<512>::new(),
                });

                // y is displaying horizontal offset something like 0-239, although
                // i've never seen zero
                // x is always reporting zero

                // draw_audio(display, match touch.event_type {
                //     crate::interface::EventType::Down => "down",
                //     crate::interface::EventType::Contact => "contact",
                //     crate::interface::EventType::Up => "up",
                // }, match touch.gesture {
                //     crate::interface::Gesture::None => "none",
                //     crate::interface::Gesture::SlideDown => "slide down",
                //     crate::interface::Gesture::SlideUp => "slide up",
                //     crate::interface::Gesture::SlideLeft => "slide left",
                //     crate::interface::Gesture::SlideRight => "slide right",
                //     crate::interface::Gesture::SingleClick => "single click",
                //     crate::interface::Gesture::DoubleClick => "double click",
                //     crate::interface::Gesture::LongPress => "long press",
                // })
            }
            AppInput::ButtonPressed => {
                let new_window = self.active_window.next();
                self.active_window = new_window;
            }
            AppInput::Tick => {
                // this just triggers a re-draw
            }
        };

        self.draw(display)?;

        Ok(())
    }

    fn draw<D, E>(&self, display: &mut D) -> Result<(), E>
    where
        D: DrawTarget<Color = DisplayColor, Error = E>,
        E: core::fmt::Debug,
    {
        match self.active_window {
            ActiveWindow::Main => {
                draw_battery(display, self.charging)?;
                draw_time(display, self.time.current_time())?;
                if let Some(media_data) = self.media.borrow() {
                    draw_audio(display, &media_data.artist, &media_data.title)?;
                }
            }
            ActiveWindow::Debug => {
                // nothing for now
                draw_bg(display)?;
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
