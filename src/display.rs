use embassy_nrf::{
    bind_interrupts,
    gpio::{Level, Output, OutputDrive},
    peripherals::{P0_02, P0_03, P0_04, P0_14, P0_18, P0_25, TWISPI1},
    spim::{self, Spim},
};
use embedded_graphics::{
    mono_font::{ascii, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::{Primitive, RgbColor},
    Drawable,
};

use crate::ble::{AppleMediaServiceData, APPLE_MEDIA_SERVICE_DATA};

const LCD_W: u16 = 240;
const LCD_H: u16 = 240;

bind_interrupts!(struct Irqs {
    SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1 => spim::InterruptHandler<TWISPI1>;
});

#[embassy_executor::task]
pub async fn task(
    dc_pin: P0_18,
    cs_pin: P0_25,
    spim: TWISPI1,
    sck_pin: P0_02,
    miso_pin: P0_04,
    mosi_pin: P0_03,
) {
    let display_spi_config = {
        let mut c = spim::Config::default();

        c.frequency = spim::Frequency::M8;
        c.mode = spim::MODE_3;

        c
    };
    let display_spi = Spim::new(spim, Irqs, sck_pin, miso_pin, mosi_pin, display_spi_config);

    let display_dc = Output::new(dc_pin, Level::Low, OutputDrive::Standard);
    let display_cs = Output::new(cs_pin, Level::Low, OutputDrive::Standard);
    let display_interface =
        display_interface_spi::SPIInterface::new(display_spi, display_dc, display_cs);

    let mut display = mipidsi::Builder::st7789(display_interface)
        .with_display_size(LCD_W, LCD_H)
        .with_orientation(mipidsi::Orientation::Portrait(false))
        .with_invert_colors(mipidsi::ColorInversion::Inverted)
        .init(&mut embassy_time::Delay, None::<Output<'static, P0_14>>)
        .unwrap();

    let backdrop_style = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
        .fill_color(embedded_graphics::pixelcolor::Rgb565::BLACK)
        .build();
    embedded_graphics::primitives::Rectangle::new(
        embedded_graphics::geometry::Point::new(0, 0),
        embedded_graphics::prelude::Size::new(LCD_W as u32, LCD_H as u32),
    )
    .into_styled(backdrop_style)
    .draw(&mut display)
    .unwrap();

    let character_style = MonoTextStyle::new(&ascii::FONT_10X20, Rgb565::WHITE);
    let text_style = embedded_graphics::text::TextStyleBuilder::new()
        .baseline(embedded_graphics::text::Baseline::Top)
        .build();

    embedded_graphics::text::Text::with_text_style(
        "PineTime",
        embedded_graphics::prelude::Point::new(10, 0),
        character_style,
        text_style,
    )
    .draw(&mut display)
    .unwrap();

    loop {
        let AppleMediaServiceData { artist, title, .. } = APPLE_MEDIA_SERVICE_DATA.wait().await;

        for (mut text, text_y_pos) in [(title.as_str(), 20), (artist.as_str(), 40)] {
            // clearing out the old text
            embedded_graphics::primitives::Rectangle::new(
                embedded_graphics::geometry::Point::new(0, text_y_pos),
                embedded_graphics::prelude::Size::new(LCD_W as u32, text_y_pos as u32),
            )
            .into_styled(backdrop_style)
            .draw(&mut display)
            .unwrap();

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
            .draw(&mut display)
            .unwrap();
        }
    }
}
