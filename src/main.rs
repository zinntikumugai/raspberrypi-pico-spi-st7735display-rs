#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embedded_graphics::mono_font::iso_8859_1::FONT_6X10;
use embedded_graphics::mono_font::iso_8859_10::FONT_7X14;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::primitives::{Arc, Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::{Baseline, Text};
use embedded_graphics::image::Image;
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_hal::delay::DelayNs;
use embedded_hal_bus::spi::ExclusiveDevice;
use panic_probe as _;
use rp2040_hal::fugit::RateExtU32;
use rp2040_hal::{self as hal, Clock};

use hal::pac;

use tinybmp::Bmp;

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;
// 画像データ（img.bmp）をバイナリに含める
const BITMAP_DATA: &[u8] = include_bytes!("./img.bmp");

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

    // SPI ピンの設定
    let sck = pins.gpio18.into_function::<hal::gpio::FunctionSpi>();
    let sda = pins.gpio19.into_function::<hal::gpio::FunctionSpi>();
    let rst = pins.gpio22.into_push_pull_output();
    let dc  = pins.gpio28.into_push_pull_output();
    let cs  = pins.gpio17.into_push_pull_output();

    // rp2040_hal の SPI 初期化（SPI0、16MHz、MODE0）
    let spi: rp2040_hal::Spi<_, _, _, 8> = hal::Spi::new(pac.SPI0, (sda, sck)).init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        16u32.MHz(),
        embedded_hal::spi::MODE_0,
    );

    // 独自の DisplaySpiDevice でラップし、st7735-lcd の要求する SpiDevice を満たす
    let spi_device = ExclusiveDevice::new_no_delay(spi, cs).unwrap();

    let mut display = st7735_lcd::ST7735::new(
        spi_device,
        dc,
        rst,
        true,   // rgb モード
        false,  // inverted モード
        128,    // 幅
        160,    // 高さ
    );

    // 必要に応じてハードリセットを行い、ディスプレイ初期化
    display.hard_reset(&mut timer).unwrap();
    display.init(&mut timer).unwrap();

    // 以下、表示内容の描画例（embedded-graphics を利用）
    let text_style_a = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(Rgb565::WHITE)
        .build();
    let text_style_b = MonoTextStyleBuilder::new()
        .font(&FONT_7X14)
        .text_color(Rgb565::GREEN)
        .build();

    let text_a = Text::with_baseline(
        "Hello World.", 
        Point::new(0, 0),
        text_style_a,
        Baseline::Top
    );
    let text_b = Text::with_baseline(
        "zin3.cc", 
        Point::new(0, 10),
        text_style_b,
        Baseline::Top
    );

    let bmp = Bmp::from_slice(BITMAP_DATA).unwrap();
    let image = Image::new(
        &bmp,
        Point::new(0, 30)
    );

    let rectangle = Rectangle::new(
        Point::new(16, 24), Size::new(32, 16)
    ).into_styled(
        PrimitiveStyleBuilder::new()
            .stroke_width(2)
            .stroke_color(Rgb565::RED)
            .fill_color(Rgb565::CYAN)
            .build(),
    );

    let ark = Arc::new(
        Point::new(60, 60),
        40,
        -30.0.deg(),
        150.0.deg()
    )
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::GREEN, 4));

    loop {
        timer.delay_ms(2000);
        display.clear(Rgb565::BLACK).unwrap();
        display.set_offset(0, 25);

        text_a.draw(&mut display).unwrap();
        text_b.draw(&mut display).unwrap();
        info!("draw text!");

        image.draw(&mut display).unwrap();
        info!("draw image");

        rectangle.draw(&mut display).unwrap();
        info!("draw rectangle");

        for i in 1..120 {
            Line::new(
                Point::new(30, 40),
                Point::new(70, i)
            )
            .into_styled(PrimitiveStyle::with_stroke(Rgb565::RED, 2))
            .draw(&mut display).unwrap();
            
        }
        info!("draw line");

        ark.draw(&mut display).unwrap();
        info!("draw arc");
    }
}
