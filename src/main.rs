#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use {defmt_rtt as _, panic_probe as _};

use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_nrf::{bind_interrupts, gpio::{Level, Output, OutputDrive}, peripherals::{P0_14, self}, spim::{self, Spim}};
use embassy_time::{Duration, Timer};
use embedded_graphics::{Drawable, prelude::{Primitive, RgbColor}, mono_font::{MonoTextStyle, ascii}, pixelcolor::Rgb565};

const LCD_W: u16 = 240;
const LCD_H: u16 = 240;

bind_interrupts!(struct Irqs {
    SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1 => spim::InterruptHandler<peripherals::TWISPI1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let led = Output::new(p.P0_14, Level::Low, OutputDrive::Standard);


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
    let display_interface = display_interface_spi::SPIInterface::new(display_spi, display_dc, display_cs);

    let mut display = mipidsi::Builder::st7789(display_interface)
        .with_display_size(LCD_W, LCD_H)
        .with_orientation(mipidsi::Orientation::Portrait(false))
        .with_invert_colors(mipidsi::ColorInversion::Inverted)
        .init(&mut embassy_time::Delay, None::<Output<'static, P0_14>>)
        .unwrap();

    let backdrop_style = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
        .fill_color(embedded_graphics::pixelcolor::Rgb565::new(0, 255, 0))
        .build();
    embedded_graphics::primitives::Rectangle::new(embedded_graphics::geometry::Point::new(0, 0), embedded_graphics::prelude::Size::new(LCD_W as u32, LCD_H as u32))
        .into_styled(backdrop_style)
        .draw(&mut display)
        .unwrap();

    embedded_graphics::text::Text::new("PineTime", embedded_graphics::prelude::Point::new(10, 10), 
        MonoTextStyle::new(&ascii::FONT_10X20, Rgb565::WHITE))
        .draw(&mut display)
        .unwrap();
    unwrap!(spawner.spawn(blinker(led, Duration::from_millis(300))));
}

#[embassy_executor::task]
async fn blinker(mut led: Output<'static, P0_14>, interval: Duration) {
    loop {
        led.set_high();
        Timer::after(interval).await;
        led.set_low();
        Timer::after(interval).await;
    }
}
