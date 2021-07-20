#![no_std]
#![no_main]

extern crate panic_halt;

use embedded_hal::spi::MODE_0;

use hifive1::hal::delay::Delay;
use hifive1::hal::prelude::*;
use hifive1::hal::spi::Spi;
use hifive1::hal::DeviceResources;

use riscv_rt::entry;

use epd_waveshare::epd2in9_v2::{Display2in9, Epd2in9};
use epd_waveshare::prelude::*;

use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    pixelcolor::BinaryColor::{Off, On},
    prelude::*,
    primitives::*,
    text::{Baseline, Text, TextStyleBuilder},
};

use embedded_text::{alignment::VerticalAlignment, TextBox};

#[entry]
fn main() -> ! {
    let dr = DeviceResources::take().unwrap();
    let p = dr.peripherals;

    // Configure clocks
    let clocks = hifive1::clock::configure(p.PRCI, p.AONCLK, 320.mhz().into());

    let mosi = dr.pins.pin3.into_iof0();
    let clk = dr.pins.pin5.into_iof0();

    let cs = dr.pins.pin23.into_output();
    let dc = dr.pins.pin22.into_output();
    let rst = dr.pins.pin21.into_output();
    let busy = dr.pins.pin20.into_floating_input();

    let mut spi = Spi::new(p.QSPI1, (mosi, (), clk), MODE_0, 8_000_000u32.hz(), clocks);

    let mut delay = Delay;

    let mut epd = Epd2in9::new(&mut spi, cs, busy, dc, rst, &mut delay).unwrap();

    let mut display = Display2in9::default();

    let _ = display.clear(Off);

    let _ = epd.update_old_frame(&mut spi, display.buffer(), &mut delay);
    let _ = epd.display_frame(&mut spi, &mut delay);

    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X12)
        .text_color(On)
        .background_color(Off)
        .build();

    let _ = TextBox::with_vertical_alignment(
        "Hello from Rust on RED-V, connected to an E-Ink display...",
        Rectangle::new(
            Point::new(1, 1),
            Size::new(epd.width() - 2, epd.height() - 2),
        ),
        style,
        VerticalAlignment::Scrolling,
    )
    .draw(&mut display);

    let _ = epd.update_and_display_new_frame(&mut spi, display.buffer(), &mut delay);

    let mut count = 0u32;

    let mut buf = [0u8; 64];

    loop {
        let string: &str = write_to::show(&mut buf, format_args!("count: {}", count)).unwrap();

        draw_text(&mut display, string, 1, 50);

        let _ = epd.update_and_display_new_frame(&mut spi, display.buffer(), &mut delay);

        count = count + 1;
    }
}

#[allow(unused)]
fn draw_text(display: &mut Display2in9, text: &str, x: i32, y: i32) {
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X12)
        .text_color(On)
        .background_color(Off)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let _ = Text::with_text_style(text, Point::new(x, y), style, text_style).draw(display);
}

// Copy paste from
// https://stackoverflow.com/questions/50200268/how-can-i-use-the-format-macro-in-a-no-std-environment
// because I was lazy...
pub mod write_to {
    use core::cmp::min;
    use core::fmt;

    pub struct WriteTo<'a> {
        buffer: &'a mut [u8],
        // on write error (i.e. not enough space in buffer) this grows beyond
        // `buffer.len()`.
        used: usize,
    }

    impl<'a> WriteTo<'a> {
        pub fn new(buffer: &'a mut [u8]) -> Self {
            WriteTo { buffer, used: 0 }
        }

        pub fn as_str(self) -> Option<&'a str> {
            if self.used <= self.buffer.len() {
                // only successful concats of str - must be a valid str.
                use core::str::from_utf8_unchecked;
                Some(unsafe { from_utf8_unchecked(&self.buffer[..self.used]) })
            } else {
                None
            }
        }
    }

    impl<'a> fmt::Write for WriteTo<'a> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            if self.used > self.buffer.len() {
                return Err(fmt::Error);
            }
            let remaining_buf = &mut self.buffer[self.used..];
            let raw_s = s.as_bytes();
            let write_num = min(raw_s.len(), remaining_buf.len());
            remaining_buf[..write_num].copy_from_slice(&raw_s[..write_num]);
            self.used += raw_s.len();
            if write_num < raw_s.len() {
                Err(fmt::Error)
            } else {
                Ok(())
            }
        }
    }

    pub fn show<'a>(buffer: &'a mut [u8], args: fmt::Arguments) -> Result<&'a str, fmt::Error> {
        let mut w = WriteTo::new(buffer);
        fmt::write(&mut w, args)?;
        w.as_str().ok_or(fmt::Error)
    }
}
