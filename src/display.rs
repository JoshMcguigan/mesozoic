use embassy_nrf::{
    bind_interrupts,
    gpio::{Level, Output, OutputDrive},
    peripherals::{self, P0_14},
    spim::{self, Spim},
};
use embedded_graphics::{
    mono_font::{ascii, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::{Primitive, RgbColor},
    Drawable,
};

const LCD_W: u16 = 240;
const LCD_H: u16 = 240;

bind_interrupts!(struct Irqs {
    SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1 => spim::InterruptHandler<peripherals::TWISPI1>;
});

#[embassy_executor::task]
pub async fn task(p: embassy_nrf::Peripherals) {
    // Turn on the backlight, then `forget` this pin to skip the drop implementation
    // which resets the configuration.
    core::mem::forget(Output::new(p.P0_14, Level::Low, OutputDrive::Standard));

    let display_spi_config = {
        let mut c = spim::Config::default();

        c.frequency = spim::Frequency::M8;
        c.mode = spim::MODE_3;

        c
    };
    let display_spi = Spim::new(
        p.TWISPI1,
        Irqs,
        p.P0_02,
        p.P0_04,
        p.P0_03,
        display_spi_config,
    );

    let display_dc = Output::new(p.P0_18, Level::Low, OutputDrive::Standard);
    let display_cs = Output::new(p.P0_25, Level::Low, OutputDrive::Standard);
    let display_interface =
        display_interface_spi::SPIInterface::new(display_spi, display_dc, display_cs);

    let mut display = mipidsi::Builder::st7789(display_interface)
        .with_display_size(LCD_W, LCD_H)
        .with_orientation(mipidsi::Orientation::Portrait(false))
        .with_invert_colors(mipidsi::ColorInversion::Inverted)
        .init(&mut embassy_time::Delay, None::<Output<'static, P0_14>>)
        .unwrap();

    let backdrop_style = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
        .fill_color(embedded_graphics::pixelcolor::Rgb565::new(0, 255, 0))
        .build();
    embedded_graphics::primitives::Rectangle::new(
        embedded_graphics::geometry::Point::new(0, 0),
        embedded_graphics::prelude::Size::new(LCD_W as u32, LCD_H as u32),
    )
    .into_styled(backdrop_style)
    .draw(&mut display)
    .unwrap();

    embedded_graphics::text::Text::new(
        "PineTime",
        embedded_graphics::prelude::Point::new(10, 10),
        MonoTextStyle::new(&ascii::FONT_10X20, Rgb565::WHITE),
    )
    .draw(&mut display)
    .unwrap();
}
