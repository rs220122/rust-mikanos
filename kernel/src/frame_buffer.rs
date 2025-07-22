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

    fn write_no_check(&mut self, x: u32, y: u32, c: &PixelColor) -> bool {
        let pixel_at: usize = 4 * (self.pixels_per_scan_line * (y as usize) + (x as usize));

        self.frame_buffer[pixel_at] = c.b;
        self.frame_buffer[pixel_at + 1] = c.g;
        self.frame_buffer[pixel_at + 2] = c.r;

        true
    }
}
