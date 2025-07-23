use crate::font::get_font;
use core::fmt::{Result, Write};
use core::ops::Add;

pub struct Vector2D<T> {
    pub x: T,
    pub y: T,
}

impl<T> Add for Vector2D<T>
where
    T: Add<Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Vector2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl<'a, 'b, T> Add<&'b Vector2D<T>> for &'a Vector2D<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = Vector2D<T>;

    fn add(self, rhs: &'b Vector2D<T>) -> Self::Output {
        Vector2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> Vector2D<T> {
    pub fn new(x: T, y: T) -> Self {
        Vector2D { x, y }
    }
}

pub enum PixelFormat {
    kPixelRGBResv8BitPerColor,
    kPixelBGRResv8BitPerColor,
}
pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub trait PixelWriter<'a> {
    fn new(
        frame_buffer_base: usize,
        pixels_per_scan_line: u32,
        horizontal_resolution: u32,
        vertical_resolution: u32,
    ) -> Self;
    fn write_no_check(&mut self, x: u32, y: u32, c: &PixelColor) -> bool;
    fn write_ascii(&mut self, x: u32, y: u32, c: u8, color: &PixelColor) {
        let font = get_font(c);
        if font.is_null() {
            return;
        }
        let font: &[u8; 16] = unsafe { core::mem::transmute(font) };
        for (dy, font_bits) in font.iter().enumerate() {
            for dx in 0..8usize {
                if (*font_bits << dx) & 0x80 != 0 {
                    self.write_no_check(x + dx as u32, y + dy as u32, color);
                }
            }
        }
    }
    fn horizontal_resolution(&self) -> u32;
    fn vertical_resolution(&self) -> u32;

    fn write_string(&mut self, x: u32, y: u32, s: &str, color: &PixelColor) {
        for (i, c) in s.chars().enumerate() {
            let i = i as u32;
            self.write_ascii(x + 8 * i, y, c as u8, color);
        }
    }
    fn is_over_pos(&self, pos: &Vector2D<u32>) -> bool {
        if pos.x > self.horizontal_resolution() {
            return true;
        }
        if pos.y > self.vertical_resolution() {
            return true;
        }
        false
    }
    fn fill_rectangle(&mut self, pos: &Vector2D<u32>, size: &Vector2D<u32>, color: &PixelColor) {
        let bottom_right = pos + size;
        if self.is_over_pos(pos) {
            return;
        }
        if self.is_over_pos(&bottom_right) {
            return;
        }

        for x in pos.x..bottom_right.x {
            for y in pos.y..bottom_right.y {
                self.write_no_check(x, y, color);
            }
        }
    }
    fn draw_rectangle(&mut self, pos: &Vector2D<u32>, size: &Vector2D<u32>, color: &PixelColor) {
        let bottom_right = pos + size;
        if self.is_over_pos(pos) {
            return;
        }
        if self.is_over_pos(&bottom_right) {
            return;
        }
        for x in pos.x..bottom_right.x {
            self.write_no_check(x, pos.y, color);
            self.write_no_check(x, bottom_right.y - 1, color);
        }
        for y in (pos.y + 1)..(bottom_right.y - 1) {
            self.write_no_check(pos.x, y, color);
            self.write_no_check(bottom_right.x - 1, y, color);
        }
    }
}
pub struct RGBResv8BitPerColorPixelWriter<'a> {
    frame_buffer: &'a mut [u8],
    pixels_per_scan_line: usize,
    pub horizontal_resolution: usize,
    pub vertical_resolution: usize,
}
pub struct BGRResv8BitPerColorPixelWriter<'a> {
    frame_buffer: &'a mut [u8],
    pixels_per_scan_line: usize,
    pub horizontal_resolution: usize,
    pub vertical_resolution: usize,
}

impl<'a> PixelWriter<'a> for RGBResv8BitPerColorPixelWriter<'a> {
    fn new(
        frame_buffer_base: usize,
        pixels_per_scan_line: u32,
        horizontal_resolution: u32,
        vertical_resolution: u32,
    ) -> Self {
        // フレームバッファのバイト数を計算
        let frame_buffer_size: usize =
            (pixels_per_scan_line as usize) * (vertical_resolution as usize) * 4;

        let frame_buffer = unsafe {
            core::slice::from_raw_parts_mut(frame_buffer_base as *mut u8, frame_buffer_size)
        };

        RGBResv8BitPerColorPixelWriter {
            frame_buffer,
            pixels_per_scan_line: pixels_per_scan_line as usize,
            horizontal_resolution: horizontal_resolution as usize,
            vertical_resolution: vertical_resolution as usize,
        }
    }
    fn horizontal_resolution(&self) -> u32 {
        self.horizontal_resolution as u32
    }
    fn vertical_resolution(&self) -> u32 {
        self.vertical_resolution as u32
    }

    fn write_no_check(&mut self, x: u32, y: u32, c: &PixelColor) -> bool {
        let pixel_at: usize = 4 * (self.pixels_per_scan_line * (y as usize) + (x as usize));

        self.frame_buffer[pixel_at] = c.r;
        self.frame_buffer[pixel_at + 1] = c.g;
        self.frame_buffer[pixel_at + 2] = c.b;

        true
    }
}

impl<'a> PixelWriter<'a> for BGRResv8BitPerColorPixelWriter<'a> {
    fn new(
        frame_buffer_base: usize,
        pixels_per_scan_line: u32,
        horizontal_resolution: u32,
        vertical_resolution: u32,
    ) -> Self {
        // フレームバッファのバイト数を計算
        let frame_buffer_size: usize =
            (pixels_per_scan_line as usize) * (vertical_resolution as usize) * 4;

        let frame_buffer = unsafe {
            core::slice::from_raw_parts_mut(frame_buffer_base as *mut u8, frame_buffer_size)
        };

        BGRResv8BitPerColorPixelWriter {
            frame_buffer,
            pixels_per_scan_line: pixels_per_scan_line as usize,
            horizontal_resolution: horizontal_resolution as usize,
            vertical_resolution: vertical_resolution as usize,
        }
    }
    fn horizontal_resolution(&self) -> u32 {
        self.horizontal_resolution as u32
    }
    fn vertical_resolution(&self) -> u32 {
        self.vertical_resolution as u32
    }

    fn write_no_check(&mut self, x: u32, y: u32, c: &PixelColor) -> bool {
        let pixel_at: usize = 4 * (self.pixels_per_scan_line * (y as usize) + (x as usize));

        self.frame_buffer[pixel_at] = c.b;
        self.frame_buffer[pixel_at + 1] = c.g;
        self.frame_buffer[pixel_at + 2] = c.r;

        true
    }
}

pub enum PixelWriterKind<'a> {
    RGB8(RGBResv8BitPerColorPixelWriter<'a>),
    BGR8(BGRResv8BitPerColorPixelWriter<'a>), // BGR8(BGRResv8BitPerColorPixelWriter<'a>), // 他の実装があるなら追加
}
pub struct Console<'a> {
    pixel_writer: PixelWriterKind<'a>,
    buf: &'a mut [&'a mut [u8]; 30],
    fg_color: PixelColor,
    bg_color: PixelColor,
    n_rows: usize,
    n_columns: usize,
    cursor_row: usize,
    cursor_columns: usize,
}

impl<'a> Console<'a> {
    pub fn new(
        buffer: &'a mut [&'a mut [u8]; 30],
        fg_color: PixelColor,
        bg_color: PixelColor,
        pixel_writer: PixelWriterKind<'a>,
    ) -> Self {
        if buffer.is_empty() {
            panic!("error");
        }
        let n_rows = buffer.len();
        let n_columns = buffer[0].len();
        for b in buffer.iter() {
            assert!(b.len() == n_columns)
        }
        Console {
            buf: buffer,
            fg_color,
            bg_color,
            n_rows,
            n_columns,
            pixel_writer,
            cursor_row: 0,
            cursor_columns: 0,
        }
    }

    fn write_bg(&mut self, x: u32, y: u32, c: Option<u8>) {
        match &mut self.pixel_writer {
            PixelWriterKind::RGB8(writer) => {
                if c.is_some() {
                    writer.write_ascii(x, y, c.unwrap(), &self.bg_color);
                    return;
                }
                writer.write_no_check(x, y, &self.bg_color);
            }
            PixelWriterKind::BGR8(writer) => {
                if c.is_some() {
                    writer.write_ascii(x, y, c.unwrap(), &self.bg_color);
                    return;
                }
                writer.write_no_check(x, y, &self.bg_color);
            }
        }
    }

    fn write_fg(&mut self, x: u32, y: u32, c: Option<u8>) {
        match &mut self.pixel_writer {
            PixelWriterKind::RGB8(writer) => {
                if c.is_some() {
                    writer.write_ascii(x, y, c.unwrap(), &self.fg_color);
                    return;
                }
                writer.write_no_check(x, y, &self.fg_color);
            }
            PixelWriterKind::BGR8(writer) => {
                if c.is_some() {
                    writer.write_ascii(x, y, c.unwrap(), &self.fg_color);
                    return;
                }
                writer.write_no_check(x, y, &self.fg_color);
            }
        }
    }

    fn new_line(&mut self) {
        self.cursor_columns = 0;
        if self.cursor_row < self.n_rows - 1 {
            self.cursor_row += 1;
        } else {
            // 一回全部バックグラウンドの色で初期化
            for row in 0..(self.n_rows - 1) {
                for column in 0..self.n_columns {
                    self.write_bg(
                        (column * 8) as u32,
                        (row * 16) as u32,
                        Some(self.buf[row][column]),
                    );
                    self.write_fg(
                        (column * 8) as u32,
                        (row * 16) as u32,
                        Some(self.buf[row + 1][column]),
                    );
                    self.buf[row][column] = self.buf[row + 1][column];
                }
            }
            for column in 0..self.n_columns {
                self.write_bg(
                    (column * 8) as u32,
                    ((self.n_rows - 1) * 16) as u32,
                    Some(self.buf[self.n_rows - 1][column]),
                );
            }
        }
    }

    pub fn put_str(&mut self, s: &str) {
        for c in s.chars() {
            let c = c as u8;
            if c == '\n' as u8 {
                self.new_line();
                continue;
            } else if self.cursor_columns < self.n_columns - 1 {
                self.write_fg(
                    (8 * self.cursor_columns) as u32,
                    (16 * self.cursor_row) as u32,
                    Some(c),
                );
                self.buf[self.cursor_row][self.cursor_columns] = c;
                self.cursor_columns += 1;
            }
        }
    }
}

impl<'a> core::fmt::Write for Console<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.put_str(s); // すでにある `put_string` を使う
        Ok(())
    }
}
