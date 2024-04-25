use arrayvec::ArrayString;
use embedded_graphics::draw_target::DrawTarget;

use crate::{
    display::{draw_audio, draw_battery, draw_bg, draw_time},
    interface::{AppInput, DisplayColor, TimeOfDay},
};

pub struct App {
    time: TimeState,
}

struct TimeState {
    ms_since_boot_when_time_last_specified: u64,
    last_specified_time: TimeOfDay,
    current_ms_since_boot: u64,
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
        draw_bg(display)?;

        Ok(Self {
            time: TimeState {
                ms_since_boot_when_time_last_specified: ms_since_boot,
                last_specified_time: TimeOfDay::default(),
                current_ms_since_boot: ms_since_boot,
            },
        })
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
        self.time.current_ms_since_boot = ms_since_boot;

        match event {
            AppInput::AppleMedia(e) => draw_audio(display, &e.artist, &e.title),
            AppInput::Battery(e) => draw_battery(display, e.charging),
            AppInput::Time(e) => {
                self.time.last_specified_time = e;
                self.time.ms_since_boot_when_time_last_specified = ms_since_boot;

                draw_time(display, self.time.current_time())
            }
            AppInput::Touch(touch) => {
                // TODO replace this with something reasonable

                use core::fmt::Write;
                let mut x = ArrayString::<3>::new();
                let mut y = ArrayString::<3>::new();
                write!(&mut x, "{}", touch.x).unwrap();
                write!(&mut y, "{}", touch.y).unwrap();
                draw_audio(display, &y, &x)

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
            // TODO re-drawing the time every tick is not necessary and leads to
            // screen flicker. Also we are re-drawing a lot more of the time
            // than actually changes second by second.
            AppInput::Tick => draw_time(display, self.time.current_time()),
        }?;

        Ok(())
    }
}
