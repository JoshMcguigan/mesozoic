use arrayvec::ArrayString;
use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_futures::select::{select, select3, Either, Either3};
use embassy_nrf::{
    bind_interrupts,
    gpio::{Level, Output, OutputDrive},
    peripherals::{P0_02, P0_03, P0_04, P0_14, P0_18, P0_25, TWISPI1},
    spim::{self, Spim},
};
use embassy_time::Timer;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    mono_font::{ascii, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::{Primitive, RgbColor},
    primitives::PrimitiveStyleBuilder,
    Drawable,
};

use core::fmt::Write;

use crate::{
    battery::{BatteryData, BATTERY_DATA},
    ble::{AppleMediaServiceData, CurrentTime, APPLE_MEDIA_SERVICE_DATA, TIME_SERVICE_DATA},
};

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
    unwrap!(Spawner::for_current_executor()
        .await
        .spawn(internal_timer()));

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
        match select3(
            APPLE_MEDIA_SERVICE_DATA.wait(),
            BATTERY_DATA.wait(),
            INTERNAL_TIME_DATA.wait(),
        )
        .await
        {
            Either3::First(AppleMediaServiceData { artist, title, .. }) => {
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
            Either3::Second(BatteryData { charging }) => {
                // The battery task immediately signals this event on startup so we
                // don't need to draw the battery un-conditionally on startup.
                draw_battery(&mut display, charging).unwrap();
            }
            Either3::Third(current_time) => {
                // The ble task signals this event on ble connection so we
                // don't need to draw the time un-conditionally on startup.
                draw_time(&mut display, current_time).unwrap();
            }
        };
    }
}

fn draw_battery<D>(display: &mut D, charging: bool) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb565>,
{
    let fill_color = match charging {
        true => Rgb565::GREEN,
        false => Rgb565::RED,
    };
    embedded_graphics::primitives::Rectangle::new(Point::new(200, 0), Size::new(32, 16))
        .into_styled(
            PrimitiveStyleBuilder::new()
                .stroke_width(2)
                .stroke_color(Rgb565::WHITE)
                .fill_color(fill_color)
                .build(),
        )
        .draw(display)
}

const TIME_NUM_CHARS: usize = 8;

fn draw_time<D, E>(display: &mut D, time: CurrentTime) -> Result<Point, E>
where
    D: DrawTarget<Color = Rgb565, Error = E>,
    E: core::fmt::Debug,
{
    // TODO factor these styles out so they aren't defined in multiple places
    let character_style = MonoTextStyle::new(&ascii::FONT_10X20, Rgb565::WHITE);
    let character_width = 10;
    let character_height = 20;
    let text_style = embedded_graphics::text::TextStyleBuilder::new()
        .baseline(embedded_graphics::text::Baseline::Top)
        .build();
    let backdrop_style = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
        .fill_color(embedded_graphics::pixelcolor::Rgb565::BLACK)
        .build();

    let mut time_string = ArrayString::<TIME_NUM_CHARS>::new();
    // This unwrap is safe because we can tell statically that we've allocated more characters
    // than this string could ever be.
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
    .draw(display)
}

// This is a super messy way of moving the clock forward.

static INTERNAL_TIME_DATA: embassy_sync::signal::Signal<
    embassy_sync::blocking_mutex::raw::ThreadModeRawMutex,
    CurrentTime,
> = embassy_sync::signal::Signal::new();

#[embassy_executor::task]
async fn internal_timer() {
    let mut time = CurrentTime::default();

    loop {
        INTERNAL_TIME_DATA.signal(time.clone());

        match select(TIME_SERVICE_DATA.wait(), Timer::after_secs(1)).await {
            Either::First(current_time) => {
                time = current_time;
            }
            Either::Second(_) => {
                if time.seconds >= 59 {
                    time.seconds = 0;

                    if time.minutes >= 59 {
                        time.minutes = 0;

                        if time.hours >= 23 {
                            time.hours = 0;
                        } else {
                            time.hours += 1;
                        }
                    } else {
                        time.minutes += 1;
                    }
                } else {
                    time.seconds += 1;
                }
            }
        }
    }
}
