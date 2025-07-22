// https://github.com/tianocore/edk2/blob/095bfacc9e52d5e7d4ef5e4a1c9bf311dce61b18/BaseTools/Source/C/Include/Protocol/GraphicsOutput.h#L178
#[repr(C)]
#[derive(Debug)]
pub struct EfiGraphicsOutputProtocol<'a> {
    reserved: [u64; 3],
    pub mode: &'a EfiGraphicsOutputProtocolMode<'a>,
}

// グラフィックに関する情報を持つ構造体
#[repr(C)]
#[derive(Debug)]
pub struct EfiGraphicsOutputProtocolMode<'a> {
    pub max_mode: u32,
    pub mode: u32,
    pub info: &'a EfiGraphicsOutputProtocolPixelInfo,
    pub size_of_info: u64,
    pub frame_buffer_base: usize,
    pub frame_buffer_size: usize,
}

// フレームバッファの情報を持つ構造体
#[repr(C)]
#[derive(Debug)]
pub struct EfiGraphicsOutputProtocolPixelInfo {
    version: u32,
    pub horizontal_resolution: u32,
    pub vertical_resolution: u32,
    pub pixel_format: EfiGraphicsPixelFormat,
    _padding0: [u32; 4],
    pub pixels_per_scan_line: u32,
}

impl EfiGraphicsOutputProtocolPixelInfo {
    pub fn get_ppixel_format(&self) -> &str {
        match self.pixel_format {
            EfiGraphicsPixelFormat::PixelRedGreenBlueReserved8BitPerColor => "PixelRGB8bit",
            EfiGraphicsPixelFormat::PixelBlueGreenRedReserved8BitPerColor => "PixelBGR8bit",
            EfiGraphicsPixelFormat::PixelBitMask => "PixelBitMask",
            EfiGraphicsPixelFormat::PixelBltOnly => "PixelBltOnly",
            EfiGraphicsPixelFormat::PixelFormatMax => "PixelFormatMax",
        }
    }
}

#[repr(i32)]
#[derive(Debug, Copy, Clone)]
pub enum EfiGraphicsPixelFormat {
    PixelRedGreenBlueReserved8BitPerColor = 0,
    PixelBlueGreenRedReserved8BitPerColor,
    PixelBitMask,
    PixelBltOnly,
    PixelFormatMax,
}
