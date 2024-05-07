use core::fmt::Write;

use arrayvec::ArrayString;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    mono_font::ascii,
    pixelcolor::WebColors,
    prelude::RgbColor,
    primitives::{Primitive, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle},
    Drawable,
};

use crate::interface::{BatteryData, DisplayColor, TimeOfDay, LCD_H, LCD_W};

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

    Ok(())
}

pub(crate) fn draw_audio<D>(display: &mut D, artist: &str, title: &str) -> Result<(), D::Error>
where
    D: DrawTarget<Color = DisplayColor>,
{
    // TODO refactor so these styles can be shared across draw functions
    let backdrop_style = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
        .fill_color(DisplayColor::BLACK)
        .build();
    let character_style = embedded_graphics::mono_font::MonoTextStyleBuilder::new()
        .font(&ascii::FONT_9X15)
        .text_color(DisplayColor::WHITE)
        .background_color(DisplayColor::BLACK)
        .build();
    let text_style = embedded_graphics::text::TextStyleBuilder::new()
        .baseline(embedded_graphics::text::Baseline::Top)
        .build();

    for (mut text, text_y_pos) in [(title, 40), (artist, 60)] {
        // Truncate the text length to fit the screen. We should do
        // something better here eventually.
        let char_width = 9;
        let char_height = 15;
        let max_chars = (LCD_W / char_width) as usize;
        if text.len() > max_chars {
            text = &text[0..max_chars];
        }

        let remaining_horizontal_space = (LCD_W - (text.len() * char_width as usize) as u16) as u32;
        let is_odd = remaining_horizontal_space % 2 != 0;
        let left_padding = (remaining_horizontal_space / 2) + if is_odd { 1 } else { 0 };
        let right_padding = remaining_horizontal_space / 2;

        // Draw over any text that might be leftover from previous draw
        // This is only strictly needed when drawing something shorter than before
        // Since for now we draw this on every screen refresh, we don't draw over the text
        // we are about to draw (and likely previously drew) or else the text will flicker.
        embedded_graphics::primitives::Rectangle::new(
            Point::new(0, text_y_pos),
            embedded_graphics::prelude::Size::new(left_padding, char_height),
        )
        .into_styled(backdrop_style)
        .draw(display)?;
        embedded_graphics::primitives::Rectangle::new(
            Point::new((LCD_W as u32 - (right_padding)) as i32, text_y_pos),
            embedded_graphics::prelude::Size::new(remaining_horizontal_space / 2, char_height),
        )
        .into_styled(backdrop_style)
        .draw(display)?;

        // writing new text
        embedded_graphics::text::Text::with_text_style(
            text,
            embedded_graphics::prelude::Point::new(left_padding as i32, text_y_pos),
            character_style,
            text_style,
        )
        .draw(display)?;
    }

    Triangle::new(
        Point::new(100, 100),
        Point::new(100, 140),
        Point::new(140, 120),
    )
    .into_styled(
        PrimitiveStyleBuilder::new()
            .stroke_width(2)
            .fill_color(DisplayColor::new(85, 255, 85))
            .stroke_color(DisplayColor::CSS_GRAY)
            .build(),
    )
    .draw(display)?;

    Ok(())
}

pub(crate) fn draw_battery<D>(display: &mut D, battery_data: &BatteryData) -> Result<(), D::Error>
where
    D: DrawTarget<Color = DisplayColor>,
{
    let color = match battery_data.charging {
        true => DisplayColor::new(85, 255, 85),
        false => DisplayColor::new(255, 85, 85),
    };
    let font = ascii::FONT_5X7;

    // The unwrap on the write! is safe because we can tell statically that we've
    // allocated enough characters to fit this string.
    const NUM_CHARS: usize = 4;
    let mut s = ArrayString::<NUM_CHARS>::new();
    write!(&mut s, "{:1.1}v", battery_data.voltage,).unwrap();

    let outline_stoke = 2;
    let width = font.character_size.width * NUM_CHARS as u32 + 2 * outline_stoke;
    embedded_graphics::primitives::Rectangle::new(
        Point::new(LCD_W as i32 - width as i32, 0),
        Size::new(width as u32, font.character_size.height + 2 * outline_stoke),
    )
    .into_styled(
        PrimitiveStyleBuilder::new()
            .stroke_width(outline_stoke)
            .stroke_alignment(StrokeAlignment::Inside)
            .stroke_color(color)
            .build(),
    )
    .draw(display)?;

    // TODO factor these styles out so they aren't defined in multiple places
    let character_style = embedded_graphics::mono_font::MonoTextStyleBuilder::new()
        .font(&ascii::FONT_5X7)
        .text_color(DisplayColor::WHITE)
        .background_color(DisplayColor::BLACK)
        .build();
    let text_style = embedded_graphics::text::TextStyleBuilder::new()
        .baseline(embedded_graphics::text::Baseline::Top)
        .build();

    embedded_graphics::text::Text::with_text_style(
        s.as_str(),
        Point::new(
            LCD_W as i32 - width as i32 + outline_stoke as i32,
            outline_stoke as i32,
        ),
        character_style,
        text_style,
    )
    .draw(display)?;

    Ok(())
}

pub(crate) fn draw_time<D, E>(display: &mut D, time: TimeOfDay) -> Result<(), E>
where
    D: DrawTarget<Color = DisplayColor, Error = E>,
    E: core::fmt::Debug,
{
    // TODO factor these styles out so they aren't defined in multiple places
    let character_style = embedded_graphics::mono_font::MonoTextStyleBuilder::new()
        .font(&ascii::FONT_7X14_BOLD)
        .text_color(DisplayColor::WHITE)
        .background_color(DisplayColor::BLACK)
        .build();
    let text_style = embedded_graphics::text::TextStyleBuilder::new()
        .baseline(embedded_graphics::text::Baseline::Top)
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

    let text_x_pos = 0;
    let text_y_pos = 0;

    embedded_graphics::text::Text::with_text_style(
        time_string.as_str(),
        Point::new(text_x_pos, text_y_pos),
        character_style,
        text_style,
    )
    .draw(display)?;

    Ok(())
}

pub(crate) fn draw_fps<D, E>(display: &mut D, fps: u32) -> Result<(), E>
where
    D: DrawTarget<Color = DisplayColor, Error = E>,
    E: core::fmt::Debug,
{
    // TODO factor these styles out so they aren't defined in multiple places
    let character_style = embedded_graphics::mono_font::MonoTextStyleBuilder::new()
        .font(&ascii::FONT_7X14)
        .text_color(DisplayColor::WHITE)
        .background_color(DisplayColor::BLACK)
        .build();
    let text_style = embedded_graphics::text::TextStyleBuilder::new()
        .baseline(embedded_graphics::text::Baseline::Top)
        .build();

    // The unwrap on the write! is safe because we can tell statically that we've
    // allocated enough characters to fit this string.
    const NUM_CHARS: usize = 8;
    let mut s = ArrayString::<NUM_CHARS>::new();
    write!(
        &mut s,
        "FPS: {:03}",
        // Limit the displayed fps value to max we can fit in three characters
        fps.min(999),
    )
    .unwrap();

    let char_height = 14;
    let text_x_pos = 0;
    let text_y_pos = 240 - char_height;

    embedded_graphics::text::Text::with_text_style(
        s.as_str(),
        Point::new(text_x_pos, text_y_pos),
        character_style,
        text_style,
    )
    .draw(display)?;

    Ok(())
}

pub(crate) fn draw_fullscreen_text<D, E>(display: &mut D, text: &str) -> Result<(), E>
where
    D: DrawTarget<Color = DisplayColor, Error = E>,
    E: core::fmt::Debug,
{
    // TODO factor these styles out so they aren't defined in multiple places
    let character_style = embedded_graphics::mono_font::MonoTextStyleBuilder::new()
        .font(&ascii::FONT_7X14)
        .text_color(DisplayColor::WHITE)
        .background_color(DisplayColor::BLACK)
        .build();
    let text_style = embedded_text::style::TextBoxStyle::default();

    let bounds = Rectangle::new(Point::zero(), Size::new(LCD_W as u32, LCD_H as u32));

    embedded_text::TextBox::with_textbox_style(text, bounds, character_style, text_style)
        .draw(display)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    extern crate std;

    use crate::{
        interface::{LCD_H, LCD_W},
        test_infra::{assert_snapshot, function_name, SimDisplay},
    };

    use super::*;

    #[test]
    fn audio() {
        let test_name = function_name!();
        let mut display = SimDisplay::new(Size::new(LCD_W as u32, LCD_H as u32));

        // First draw long strings, then shorter ones, to show we properly clear
        // out the old text.
        draw_audio(&mut display, "long artist", "long title").unwrap();
        draw_audio(&mut display, "artist", "title").unwrap();

        assert_snapshot(test_name, display);
    }

    #[test]
    fn fullscreen_text() {
        let test_name = function_name!();
        let mut display = SimDisplay::new(Size::new(LCD_W as u32, LCD_H as u32));

        // The expected behavior here is to display as much content as can fit on the
        // display. Any content which doesn't fit will overflow off the bottom.
        draw_fullscreen_text(
            &mut display,
            "1 first line
2 really long line that needs to split to new line
3 more text wow so much text wrapping wrapping wrapping
4 all the way down
5 more text wow so much text wrapping wrapping wrapping
6 all the way down
7 more text wow so much text wrapping wrapping wrapping
8 all the way down
9 more text wow so much text wrapping wrapping wrapping
0 all the way down
a more text wow so much text wrapping wrapping wrapping
b all the way down
c more text wow so much text wrapping wrapping wrapping
d all the way down
e more text wow so much text wrapping wrapping wrapping",
        )
        .unwrap();

        assert_snapshot(test_name, display);
    }
}
