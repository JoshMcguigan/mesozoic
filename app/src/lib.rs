#![no_std]

use core::fmt::Write;

use arrayvec::ArrayString;

use embedded_graphics::{
    mono_font::{ascii, MonoTextStyle},
    prelude::*,
    primitives::PrimitiveStyleBuilder,
};

pub type DisplayColor = embedded_graphics::pixelcolor::Rgb565;

pub const LCD_W: u16 = 240;
pub const LCD_H: u16 = 240;

pub struct App {
    time: TimeState,
}

pub struct TimeState {
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

pub enum AppInput {
    AppleMedia(AppleMediaServiceData),
    Battery(BatteryData),
    Time(TimeOfDay),
    /// The platform should provide this input at the rate requested by the app.
    ///
    /// TODO see future AppOutput::TickRate
    /// app will request higher tick rate when the display/backlight is on, for example
    Tick,
}

pub struct AppleMediaServiceData {
    pub artist: AppleMediaServiceString,
    pub album: AppleMediaServiceString,
    pub title: AppleMediaServiceString,
}
const ATT_PAYLOAD_MAX_LEN: usize = 512;
pub type AppleMediaServiceString = arrayvec::ArrayString<ATT_PAYLOAD_MAX_LEN>;

pub struct BatteryData {
    pub charging: bool,
}

impl App {
    pub fn init<D, E>(display: &mut D, ms_since_boot: u64) -> Result<Self, D::Error>
    where
        D: DrawTarget<Color = DisplayColor, Error = E>,
        E: core::fmt::Debug,
    {
        let backdrop_style = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
            .fill_color(embedded_graphics::pixelcolor::Rgb565::BLACK)
            .build();
        embedded_graphics::primitives::Rectangle::new(
            embedded_graphics::geometry::Point::new(0, 0),
            embedded_graphics::prelude::Size::new(LCD_W as u32, LCD_H as u32),
        )
        .into_styled(backdrop_style)
        .draw(display)
        .unwrap();

        let character_style = MonoTextStyle::new(&ascii::FONT_10X20, DisplayColor::WHITE);
        let text_style = embedded_graphics::text::TextStyleBuilder::new()
            .baseline(embedded_graphics::text::Baseline::Top)
            .build();

        embedded_graphics::text::Text::with_text_style(
            "PineTime",
            embedded_graphics::prelude::Point::new(10, 0),
            character_style,
            text_style,
        )
        .draw(display)
        .unwrap();

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
            // TODO re-drawing the time every tick is not necessary and leads to
            // screen flicker. Also we are re-drawing a lot more of the time
            // than actually changes second by second.
            AppInput::Tick => draw_time(display, self.time.current_time()),
        }?;

        Ok(())
    }
}

fn draw_audio<D>(display: &mut D, artist: &str, title: &str) -> Result<(), D::Error>
where
    D: DrawTarget<Color = DisplayColor>,
{
    // TODO refactor so these styles can be shared across draw functions
    let backdrop_style = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
        .fill_color(embedded_graphics::pixelcolor::Rgb565::BLACK)
        .build();
    let character_style = MonoTextStyle::new(&ascii::FONT_10X20, DisplayColor::WHITE);
    let text_style = embedded_graphics::text::TextStyleBuilder::new()
        .baseline(embedded_graphics::text::Baseline::Top)
        .build();

    for (mut text, text_y_pos) in [(title, 20), (artist, 40)] {
        // clearing out the old text
        embedded_graphics::primitives::Rectangle::new(
            embedded_graphics::geometry::Point::new(0, text_y_pos),
            embedded_graphics::prelude::Size::new(LCD_W as u32, text_y_pos as u32),
        )
        .into_styled(backdrop_style)
        .draw(display)?;

        // Truncate the text length to fit the screen. We should do
        // something better here eventually.
        let char_width = 10;
        let max_chars = (LCD_W / char_width) as usize;
        if text.len() > max_chars {
            text = &text[0..max_chars];
        }

        // writing new text
        embedded_graphics::text::Text::with_text_style(
            text,
            embedded_graphics::prelude::Point::new(10, text_y_pos),
            character_style,
            text_style,
        )
        .draw(display)?;
    }

    Ok(())
}

fn draw_battery<D>(display: &mut D, charging: bool) -> Result<(), D::Error>
where
    D: DrawTarget<Color = DisplayColor>,
{
    let fill_color = match charging {
        true => DisplayColor::new(85, 255, 85),
        false => DisplayColor::new(255, 85, 85),
    };
    let width = 32;
    embedded_graphics::primitives::Rectangle::new(
        Point::new(LCD_W as i32 - width, 0),
        Size::new(width as u32, 16),
    )
    .into_styled(
        PrimitiveStyleBuilder::new()
            .stroke_width(2)
            .stroke_color(DisplayColor::WHITE)
            .fill_color(fill_color)
            .build(),
    )
    .draw(display)
}

#[derive(Default, Clone)]
pub struct TimeOfDay {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
}

fn draw_time<D, E>(display: &mut D, time: TimeOfDay) -> Result<(), E>
where
    D: DrawTarget<Color = DisplayColor, Error = E>,
    E: core::fmt::Debug,
{
    // TODO factor these styles out so they aren't defined in multiple places
    let character_style = MonoTextStyle::new(&ascii::FONT_10X20, DisplayColor::WHITE);
    let character_width = 10;
    let character_height = 20;
    let text_style = embedded_graphics::text::TextStyleBuilder::new()
        .baseline(embedded_graphics::text::Baseline::Top)
        .build();
    let backdrop_style = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
        .fill_color(embedded_graphics::pixelcolor::Rgb565::BLACK)
        .build();

    // The unwrap on the write! is safe because we can tell statically that we've
    // allocated enough characters to fit this string.
    const TIME_NUM_CHARS: usize = 8;
    let mut time_string = ArrayString::<TIME_NUM_CHARS>::new();
    write!(
        &mut time_string,
        "{:02}:{:02}:{:02}",
        time.hours, time.minutes, time.seconds
    )
    .unwrap();

    let text_x_pos = 10;
    let text_y_pos = 200;

    // clearing out the old text
    embedded_graphics::primitives::Rectangle::new(
        embedded_graphics::geometry::Point::new(text_x_pos, text_y_pos),
        embedded_graphics::prelude::Size::new(
            (TIME_NUM_CHARS * character_width) as u32,
            character_height,
        ),
    )
    .into_styled(backdrop_style)
    .draw(display)
    .unwrap();

    embedded_graphics::text::Text::with_text_style(
        time_string.as_str(),
        Point::new(text_x_pos, text_y_pos),
        character_style,
        text_style,
    )
    .draw(display)?;

    Ok(())
}
