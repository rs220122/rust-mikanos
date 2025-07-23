#![no_std]
#![no_main]
use core::arch::asm;
use core::fmt::Result;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::null_mut;
use core::slice;
use core::writeln;
use kernel::graphics::Vector2D;
use kernel::graphics::{
    BGRResv8BitPerColorPixelWriter, Console, PixelColor, PixelWriter, PixelWriterKind,
    RGBResv8BitPerColorPixelWriter,
};
use kernel::pci;

const MOUSE_CURSOR_WIDTH: usize = 15;
const MOUSE_CURSOR_HEIGHT: usize = 24;
const A: u8 = '@' as u8;
const D: u8 = '.' as u8;
const MOUSE_CURSOR_SHAPE: [[u8; MOUSE_CURSOR_WIDTH + 1]; MOUSE_CURSOR_HEIGHT] = [
    [A, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [A, A, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [A, D, A, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [A, D, D, D, A, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [A, D, D, D, D, A, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [A, D, D, D, D, D, A, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [A, D, D, D, D, D, D, A, 0, 0, 0, 0, 0, 0, 0, 0],
    [A, D, D, D, D, D, D, D, A, 0, 0, 0, 0, 0, 0, 0],
    [A, D, D, D, D, D, D, D, D, A, 0, 0, 0, 0, 0, 0],
    [A, D, D, D, D, D, D, D, D, D, A, 0, 0, 0, 0, 0],
    [A, D, D, D, D, D, D, D, D, D, D, A, 0, 0, 0, 0],
    [A, D, D, D, D, D, D, D, D, D, D, D, A, 0, 0, 0],
    [A, D, D, D, D, D, D, D, D, D, D, D, D, A, 0, 0],
    [A, D, D, D, D, D, D, D, D, D, D, D, D, D, A, 0],
    [A, D, D, D, D, D, D, A, A, A, A, A, A, A, A, A],
    [A, D, D, D, D, D, D, A, 0, 0, 0, 0, 0, 0, 0, 0],
    [A, D, D, D, D, A, A, D, A, 0, 0, 0, 0, 0, 0, 0],
    [A, D, D, D, A, 0, A, D, A, 0, 0, 0, 0, 0, 0, 0],
    [A, D, D, A, 0, 0, 0, A, D, A, 0, 0, 0, 0, 0, 0],
    [A, D, A, 0, 0, 0, 0, A, D, A, 0, 0, 0, 0, 0, 0],
    [A, A, 0, 0, 0, 0, 0, 0, A, D, A, 0, 0, 0, 0, 0],
    [A, 0, 0, 0, 0, 0, 0, 0, A, D, A, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, A, D, A, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, A, A, A, 0, 0, 0, 0],
];

fn _write_mouse<'a>(pixel_writer: &mut impl PixelWriter<'a>) {
    let edge_color = PixelColor { r: 0, g: 0, b: 0 };
    let fill_color = PixelColor {
        r: 255,
        g: 255,
        b: 255,
    };
    for dy in 0..MOUSE_CURSOR_HEIGHT {
        for dx in 0..MOUSE_CURSOR_WIDTH {
            if MOUSE_CURSOR_SHAPE[dy][dx] == A {
                pixel_writer.write_no_check(200 + dx as u32, 100 + dy as u32, &edge_color);
            } else if MOUSE_CURSOR_SHAPE[dy][dx] == D {
                pixel_writer.write_no_check(200 + dx as u32, 100 + dy as u32, &fill_color);
            }
        }
    }
}

fn write_mouse(pixel_writer: &mut PixelWriterKind) {
    match pixel_writer {
        PixelWriterKind::RGB8(writer) => {
            _write_mouse(writer);
        }
        PixelWriterKind::BGR8(writer) => _write_mouse(writer),
    }
}

fn fill_rectangle(
    pixel_writer: &mut PixelWriterKind,
    pos: &Vector2D<u32>,
    size: &Vector2D<u32>,
    color: &PixelColor,
) {
    match pixel_writer {
        PixelWriterKind::RGB8(writer) => {
            writer.fill_rectangle(pos, size, color);
        }
        PixelWriterKind::BGR8(writer) => {
            writer.fill_rectangle(pos, size, color);
        }
    }
}

fn draw_rectangle(
    pixel_writer: &mut PixelWriterKind,
    pos: &Vector2D<u32>,
    size: &Vector2D<u32>,
    color: &PixelColor,
) {
    match pixel_writer {
        PixelWriterKind::RGB8(writer) => {
            writer.draw_rectangle(pos, size, color);
        }
        PixelWriterKind::BGR8(writer) => {
            writer.draw_rectangle(pos, size, color);
        }
    }
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
            let pixel_color = PixelColor { r: 0, g: 0, b: 0 };
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

    // rectangleを描く
    let desktop_bg_color = PixelColor {
        r: 45,
        g: 118,
        b: 237,
    };
    let desktop_fg_color = PixelColor {
        r: 255,
        g: 255,
        b: 255,
    };
    fill_rectangle(
        &mut pixel_writer,
        &Vector2D::new(0, 0),
        &Vector2D::new(horizontal_resolution, vertical_resolution - 50),
        &desktop_bg_color,
    );
    fill_rectangle(
        &mut pixel_writer,
        &Vector2D::new(0, vertical_resolution - 50),
        &Vector2D::new(horizontal_resolution, 50),
        &PixelColor { r: 1, g: 8, b: 17 },
    );
    fill_rectangle(
        &mut pixel_writer,
        &Vector2D::new(0, vertical_resolution - 50),
        &Vector2D::new(horizontal_resolution / 5, 50),
        &PixelColor {
            r: 80,
            g: 80,
            b: 80,
        },
    );
    draw_rectangle(
        &mut pixel_writer,
        &Vector2D::new(10, vertical_resolution - 40),
        &Vector2D::new(30, 30),
        &PixelColor {
            r: 255,
            g: 255,
            b: 255,
        },
    );

    write_mouse(&mut pixel_writer);

    let mut buffer = [[0u8; 75]; 30];
    let mut buf: [&mut [u8]; 30] = {
        let mut tmp: [core::mem::MaybeUninit<&mut [u8]>; 30] =
            unsafe { core::mem::MaybeUninit::uninit().assume_init() };

        // 生ポインタで安全に借用を回避
        let buffer_ptr = buffer.as_mut_ptr();

        for i in 0..30 {
            unsafe {
                let row_ptr = buffer_ptr.add(i); // pointer to [u8; COLS]
                tmp[i] = core::mem::MaybeUninit::new(&mut (*row_ptr)[..]);
            }
        }

        // 初期化済みに変換
        unsafe { core::mem::transmute::<_, [&mut [u8]; 30]>(tmp) }
    };

    // 2. それを &mut [&mut [u8]] に変換
    let mut console = Console::new(&mut buf, desktop_fg_color, desktop_bg_color, pixel_writer);
    writeln!(console, "Welcome to MikanOS!");

    // PCIを読み込む
    let res = pci::scan_all_bus();
    if let Err(error) = res {
        writeln!(console, "PCI Deivce Scan Failed. status: {error:?}");
    } else {
        writeln!(console, "PCI Device scan success");
    }
    unsafe {
        // PCIデバイスを表示
        for i in 0..pci::NUM_DEVICES {
            let dev = pci::DEVICES[i];
            let vendor_id = pci::read_vendor_id(dev.bus, dev.device, dev.function);
            let class_code = pci::read_class_code(dev.bus, dev.device, dev.function);
            writeln!(
                console,
                "{}.{}.{}: vend {:04X}, class {:08X} head: {:02X}",
                dev.bus, dev.device, dev.function, vendor_id, class_code, dev.header_type
            );
        }
    }

    // Intel製を優先してxHCを探す
    // let mut xhc_dev: *mut pci::Device = core::ptr::null_mut();
    // unsafe {
    //     for i in 0..pci::NUM_DEVICES {
    //         let dev = pci::DEVICES[i];
    //     }
    // }
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
