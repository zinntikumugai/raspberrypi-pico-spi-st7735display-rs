#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embedded_graphics::mono_font::iso_8859_1::FONT_6X10;
use embedded_graphics::mono_font::iso_8859_10::FONT_7X14;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::text::{Baseline, Text};
use panic_probe as _;
use rp2040_hal::fugit::RateExtU32;
use rp2040_hal::{self as hal, Clock};

use hal::pac;

use embedded_graphics::image::{Image, ImageRaw, ImageRawLE};
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::Rgb565;

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;


#[rp2040_hal::entry]
fn main() -> ! {
    info!("Program start!");
    let mut pac = pac::Peripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut timer = rp2040_hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let sio = hal::Sio::new(pac.SIO);

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );


    let sck = pins.gpio18.into_function::<hal::gpio::FunctionSpi>();
    let sda = pins.gpio19.into_function::<hal::gpio::FunctionSpi>();
    let rst = pins.gpio22.into_push_pull_output();
    let dc = pins.gpio28.into_push_pull_output();

    let spi: rp2040_hal::Spi<_, _, _, 8> = hal::Spi::new(pac.SPI0, (sda, sck)).init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        16u32.MHz(),
        embedded_hal::spi::MODE_0,
    );

    let mut display = st7735_lcd::ST7735::new(spi, dc, rst, true, false, 160, 128);

    display.init(&mut timer).unwrap();

    display.clear(Rgb565::BLACK).unwrap();
    display.set_offset(0, 25);

    let text_style_a = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(Rgb565::WHITE)
        .build();
    let text_style_b = MonoTextStyleBuilder::new()
        .font(&FONT_7X14)
        .text_color(Rgb565::GREEN)
        .build();

    Text::with_baseline(
        "Hello World.", 
        Point::new(0, 0),
        text_style_a,
        Baseline::Top
    ).draw(&mut display).unwrap();
    Text::with_baseline(
        "zin3.cc", 
        Point::new(0, 10),
        text_style_b,
        Baseline::Top
    ).draw(&mut display).unwrap();

    info!("draw text!");

    loop {
    }
}
