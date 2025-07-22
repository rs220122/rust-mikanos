#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;
use core::slice;

use kernel::frame_buffer::{
    BGRResv8BitPerColorPixelWriter, PixelColor, PixelWriter, RGBResv8BitPerColorPixelWriter,
};

pub enum PixelWriterKind<'a> {
    RGB8(RGBResv8BitPerColorPixelWriter<'a>),
    BGR8(BGRResv8BitPerColorPixelWriter<'a>), // BGR8(BGRResv8BitPerColorPixelWriter<'a>), // 他の実装があるなら追加
}
#[unsafe(no_mangle)]
extern "win64" fn KernelMain(
    frame_buffer_base: usize,
    pixels_per_scan_line: u32,
    horizontal_resolution: u32,
    vertical_resolution: u32,
    pixel_format: i32,
) -> usize {
    // フレームバッファの配列を獲得する。フレームバッファの一つのピクセルは、u32で表現される。
    // そのため、vram_addrをu32のポインタにキャスト

    let mut pixel_writer = if pixel_format == 0 {
        PixelWriterKind::RGB8(RGBResv8BitPerColorPixelWriter::new(
            frame_buffer_base,
            pixels_per_scan_line,
            horizontal_resolution,
            vertical_resolution,
        ))
    } else if pixel_format == 1 {
        PixelWriterKind::BGR8(BGRResv8BitPerColorPixelWriter::new(
            frame_buffer_base,
            pixels_per_scan_line,
            horizontal_resolution,
            vertical_resolution,
        ))
    } else {
        panic!("unimplemented color format")
    };

    for x in 0..horizontal_resolution {
        for y in 0..vertical_resolution {
            let pixel_color = PixelColor {
                r: 255,
                g: 255,
                b: 255,
            };
            match &mut pixel_writer {
                PixelWriterKind::RGB8(writer) => {
                    writer.write_no_check(x, y, &pixel_color);
                }
                PixelWriterKind::BGR8(writer) => {
                    writer.write_no_check(x, y, &pixel_color);
                }
            }
        }
    }

    let pixel_color = PixelColor { r: 0, g: 255, b: 0 };

    for x in 0..200 {
        for y in 0..100 {
            match &mut pixel_writer {
                PixelWriterKind::RGB8(writer) => {
                    writer.write_no_check(x, y, &pixel_color);
                }
                PixelWriterKind::BGR8(writer) => {
                    writer.write_no_check(x, y, &pixel_color);
                }
            }
        }
    }
    let color = PixelColor { r: 0, g: 0, b: 0 };
    match &mut pixel_writer {
        PixelWriterKind::RGB8(writer) => {
            writer.write_ascii(50, 50, 65, &color);
        }
        PixelWriterKind::BGR8(writer) => {
            writer.write_ascii(50, 50, 65, &color);
        }
    }
    match &mut pixel_writer {
        PixelWriterKind::RGB8(writer) => {
            writer.write_ascii(58, 50, 'A' as u8, &color);
        }
        PixelWriterKind::BGR8(writer) => {
            writer.write_ascii(58, 50, 'A' as u8, &color);
        }
    }

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Enter an infinite loop
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
