use core::fmt::Write;

use arrayvec::ArrayString;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    mono_font::{ascii, MonoTextStyle},
    primitives::{Primitive, PrimitiveStyleBuilder},
    prelude::RgbColor,
    Drawable,
};

use crate::interface::{DisplayColor, TimeOfDay, LCD_H, LCD_W};

pub(crate) fn draw_bg<D>(display: &mut D) -> Result<(), D::Error>
where
    D: DrawTarget<Color = DisplayColor>,
{
    let backdrop_style = PrimitiveStyleBuilder::new()
        .fill_color(DisplayColor::BLACK)
        .build();
    embedded_graphics::primitives::Rectangle::new(
        embedded_graphics::geometry::Point::new(0, 0),
        embedded_graphics::prelude::Size::new(LCD_W as u32, LCD_H as u32),
    )
    .into_styled(backdrop_style)
    .draw(display)?;

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
    .draw(display)?;

    Ok(())
}

pub(crate) fn draw_audio<D>(display: &mut D, artist: &str, title: &str) -> Result<(), D::Error>
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

pub(crate) fn draw_battery<D>(display: &mut D, charging: bool) -> Result<(), D::Error>
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

pub(crate) fn draw_time<D, E>(display: &mut D, time: TimeOfDay) -> Result<(), E>
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
        .fill_color(DisplayColor::BLACK)
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
