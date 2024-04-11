use ahora_app::{LCD_H, LCD_W};
use embassy_nrf::{
    bind_interrupts,
    gpio::{Level, Output, OutputDrive},
    peripherals::{P0_02, P0_03, P0_04, P0_14, P0_18, P0_25, TWISPI1},
    spim::{self, Spim},
};

bind_interrupts!(struct Irqs {
    SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1 => spim::InterruptHandler<TWISPI1>;
});

pub type SpiDisplay = mipidsi::Display<
    display_interface_spi::SPIInterface<
        Spim<'static, TWISPI1>,
        Output<'static, P0_18>,
        Output<'static, P0_25>,
    >,
    mipidsi::models::ST7789,
    Output<'static, P0_14>,
>;

pub fn create(
    dc_pin: P0_18,
    cs_pin: P0_25,
    spim: TWISPI1,
    sck_pin: P0_02,
    miso_pin: P0_04,
    mosi_pin: P0_03,
) -> SpiDisplay {
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

    // This unwrap is safe, because we pass a None in for the RST pin.
    mipidsi::Builder::st7789(display_interface)
        .with_display_size(LCD_W, LCD_H)
        .with_orientation(mipidsi::Orientation::Portrait(false))
        .with_invert_colors(mipidsi::ColorInversion::Inverted)
        .init(&mut embassy_time::Delay, None::<Output<'static, P0_14>>)
        .unwrap()
}
