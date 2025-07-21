#![no_std]
#![no_main]

use core::arch::asm;

use core::fmt::Write;
use core::mem::size_of;
use core::panic::PanicInfo;
use core::ptr::null_mut;
use core::slice;
use core::writeln;
// pub mod memory_map_holder;
// pub mod uefi;
// mod uefi_alloc;

use bootloader::memory_map_holder::MemoryMapHolder;
use bootloader::stack::BufWriter;
use bootloader::uefi::file::{EfiFileInfo, EfiFileProtocol, EfiSimpleFileSystemProtocol};
use bootloader::uefi::graphics::EfiGraphicsOutputProtocol;
use bootloader::uefi::open_gop;
use bootloader::uefi::text::EfiSimpleTextOutputProtocolWriter;
use bootloader::uefi::types::{
    EFI_FILE_INFO_GUID, EFI_FILE_MODE_CREATE, EFI_FILE_MODE_READ, EFI_FILE_MODE_WRITE,
    EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID, EFI_LOADED_IMAGE_PROTOCOL_GUID,
    EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_GUID, EfiGuid, EfiHandle, EfiLocateSearchType, EfiStatus,
    EfiVoid, Error,
};
use bootloader::uefi::{EfiLoadedImageProtocol, EfiSystemTable, locate_graphic_protocol};

type EntryPointType = extern "C" fn() -> ();
const MEMMAP_PATH: &[u16; 12] = &[
    (b'\\' as u16),
    (b'm' as u16),
    (b'e' as u16),
    (b'm' as u16),
    (b'm' as u16),
    (b'a' as u16),
    (b'p' as u16),
    (b'.' as u16),
    (b't' as u16),
    (b'x' as u16),
    (b't' as u16),
    (b'\0' as u16), // NULL終端
];

const KERNEL_PATH: &[u16; 12] = &[
    (b'\\' as u16),
    (b'k' as u16),
    (b'e' as u16),
    (b'r' as u16),
    (b'n' as u16),
    (b'e' as u16),
    (b'l' as u16),
    (b'.' as u16),
    (b'e' as u16),
    (b'l' as u16),
    (b'f' as u16),
    (b'\0' as u16),
];

fn open_root_dir(
    image_handle: EfiHandle,
    efi_system_table: &EfiSystemTable,
    root: *mut *mut EfiFileProtocol,
) -> EfiStatus {
    let mut output_writer = EfiSimpleTextOutputProtocolWriter::new(efi_system_table.con_out());
    // ファイルを開いてみる
    let mut loaded_image: *mut EfiLoadedImageProtocol = null_mut::<EfiLoadedImageProtocol>();
    let mut fs: *mut EfiSimpleFileSystemProtocol = null_mut::<EfiSimpleFileSystemProtocol>();

    let status = efi_system_table.boot_services.open_protocol(
        image_handle,
        &EFI_LOADED_IMAGE_PROTOCOL_GUID,
        &mut loaded_image as *mut *mut EfiLoadedImageProtocol as *mut *mut EfiVoid,
        image_handle,
        null_mut::<u64>() as u64,
    );

    if status != EfiStatus::Success {
        output_writer.write_str("Failed to open loaded image protocol");
        return status;
    }

    let loaded_image = unsafe { &*loaded_image };
    let status = efi_system_table.boot_services.open_protocol(
        loaded_image.device_handle,
        &EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_GUID,
        &mut fs as *mut *mut EfiSimpleFileSystemProtocol as *mut *mut EfiVoid,
        image_handle,
        null_mut::<u64>() as u64,
    );
    if status != EfiStatus::Success {
        output_writer.write_str("Failed to open simple file system protocol");
        return status;
    }
    output_writer.write_str("successfully opened simple file system protocol status: ");
    output_writer.write_str(status.to_string());
    output_writer.write_str("\n");

    // fsをEfiSimpleFileSystemProtocolにキャスト
    let fs: &EfiSimpleFileSystemProtocol = unsafe { &*fs };

    // ルートディレクトリを開く
    output_writer.write_str("Opening root directory...\n");
    let status = fs.open_volume(root);
    if status != EfiStatus::Success {
        output_writer.write_str("Failed to open root directory");
        return status;
    }
    output_writer.write_str("Successfully opened root directory status: ");
    output_writer.write_str(status.to_string());
    output_writer.write_str("\n");
    return status;
}

fn get_memmap_file(
    efi_system_table: &EfiSystemTable,
    root: &EfiFileProtocol,
    memmap_file: &mut *mut EfiFileProtocol,
) -> EfiStatus {
    let mut output_writer = EfiSimpleTextOutputProtocolWriter::new(efi_system_table.con_out());

    let status = root.open(
        memmap_file,
        MEMMAP_PATH,
        EFI_FILE_MODE_READ | EFI_FILE_MODE_WRITE | EFI_FILE_MODE_CREATE,
        0,
    );

    if status != EfiStatus::Success {
        output_writer.write_str("Failed to open memory map file\n");
        return status;
    }
    output_writer.write_str("Successfully opened memory map file\n");

    return status;
}

fn get_kernel_file(
    efi_system_table: &EfiSystemTable,
    root: &EfiFileProtocol,
    kernel_file: &mut *mut EfiFileProtocol,
) -> EfiStatus {
    let mut output_writer = EfiSimpleTextOutputProtocolWriter::new(efi_system_table.con_out());

    let status = root.open(
        kernel_file,
        KERNEL_PATH,
        EFI_FILE_MODE_READ | EFI_FILE_MODE_WRITE | EFI_FILE_MODE_CREATE,
        0,
    );

    if status != EfiStatus::Success {
        output_writer.write_str("Failed to open kernel.elf file\n");
        return status;
    }
    output_writer.write_str("Successfully opened kernel.elf file\n");
    return status;
}

fn write_to_memmap_file(memmap_file: &EfiFileProtocol, memory_map: &MemoryMapHolder) {
    let mut temp_buffer = [0u8; 1024];
    let mut buf_writer = BufWriter::new(&mut temp_buffer);

    let mem_buffer = memory_map.memory_map_buffer.as_ptr() as usize;
    let map_size = memory_map.memory_map_size;
    let _ =
        memmap_file.write_str("Index, Type, Type(name), PhysicalStart, NumberOfPages, Attribute\n");
    let _ = writeln!(
        buf_writer,
        "map->buffer = {mem_buffer:0>8X}, map->map_size = {map_size:0>8X}"
    );
    let _ = memmap_file.write_str(buf_writer.as_str().expect(""));
    buf_writer.flush();

    for (i, mem_descriptor) in memory_map.iter().enumerate() {
        let mem_type = mem_descriptor.memory_type as i64;
        let mem_type_str = mem_descriptor.get_memory_type_str();
        let physical_start = mem_descriptor.physical_start;
        let num_of_pages = mem_descriptor.number_of_pages;
        let attribute = mem_descriptor.attribute & 0xfffff;
        let _ = writeln!(
            buf_writer,
            "{i}, {mem_type:X}, {mem_type_str}, {physical_start:0>8X}, {num_of_pages:X}, {attribute:X}"
        );
        let _ = memmap_file.write_str(buf_writer.as_str().expect(""));
        buf_writer.flush();
    }
}

#[unsafe(no_mangle)]
/// The entry point of the bootloader
pub extern "C" fn efi_main(
    image_handle: EfiHandle,
    efi_system_table: &EfiSystemTable,
) -> EfiStatus {
    let mut console_buffer = [0u8; 1024];
    let mut buf_writer = BufWriter::new(&mut console_buffer);

    // con_outで出力
    let con_out = efi_system_table.con_out();
    let mut output_writer = EfiSimpleTextOutputProtocolWriter::new(con_out);

    let gop = open_gop(image_handle, efi_system_table).unwrap();
    let vram_addr: usize = gop.mode.frame_buffer_base;
    let vram_byte_size: usize = gop.mode.frame_buffer_size;
    let horizontal_resolution = gop.mode.info.horizontal_resolution;
    let vertical_resolution = gop.mode.info.vertical_resolution;
    let pixel_format = gop.mode.info.get_ppixel_format();
    let pixels_per_scan_line = gop.mode.info.pixels_per_scan_line;
    let _ = writeln!(
        buf_writer,
        "Resolution: {horizontal_resolution}x{vertical_resolution}, PixelFormat: {pixel_format}, {pixels_per_scan_line} p/l "
    );
    let _ = writeln!(
        buf_writer,
        "frame_buffer_base: 0x{vram_addr:0>8X}, byte size: {vram_byte_size:X}"
    );
    output_writer.write_str(buf_writer.as_str().unwrap());
    buf_writer.flush();

    // フレームバッファの配列を獲得する。フレームバッファの一つのピクセルは、u32で表現される。
    // そのため、vram_addrをu32のポインタにキャスト
    let vram_byte_size: usize =
        vertical_resolution as usize * pixels_per_scan_line as usize * 24usize;
    let vram = unsafe {
        slice::from_raw_parts_mut(vram_addr as *mut u32, vram_byte_size / size_of::<u32>())
    };
    for e in vram {
        *e = 0xffffff;
    }

    // メモリマップを取得
    let mut memory_map = MemoryMapHolder::new();
    let status = efi_system_table
        .boot_services
        .get_memory_map(&mut memory_map);
    if status != EfiStatus::Success {
        output_writer.write_str("Failed to get memory map");
        panic!("Failed to get memory map: {:?}", status);
    }
    // メモリマップの情報を表示

    // ルートディレクトリを取得
    let mut root: *mut EfiFileProtocol = null_mut::<EfiFileProtocol>();
    let status = open_root_dir(image_handle, efi_system_table, &mut root);
    if status != EfiStatus::Success {
        output_writer.write_str("Failed to open root directory");
        panic!("Failed to open root directory: {:?}", status);
    }
    let root = unsafe { &*root };

    // memmapファイルを開く
    let mut memmap_file: *mut EfiFileProtocol = null_mut::<EfiFileProtocol>();
    let status = get_memmap_file(efi_system_table, root, &mut memmap_file);
    if status != EfiStatus::Success {
        output_writer.write_str("Failed to get memory map file");
        panic!("Failed to get memory map file: {:?}", status);
    }
    // memmap_fileをEfiFileProtocolにキャスト
    let memmap_file = unsafe { &*memmap_file };

    // メモリマップをファイルに書き込む
    let _ = memmap_file.write_str("Test memory map file\n");
    write_to_memmap_file(&memmap_file, &memory_map);

    // memmapファイルを閉じる
    let status = memmap_file.close();
    if status != EfiStatus::Success {
        output_writer.write_str("Failed to close memory map file");
        panic!("Failed to close memory map file: {:?}", status);
    }
    output_writer.write_str("Successfully closed memory map file status: ");
    output_writer.write_str(status.to_string());
    output_writer.write_str("\n");

    // カーネルを展開する
    let mut kernel_file: *mut EfiFileProtocol = null_mut::<EfiFileProtocol>();
    let status = get_kernel_file(efi_system_table, root, &mut kernel_file);
    if status != EfiStatus::Success {
        output_writer.write_str("Failed to open kernel.elf");
        panic!("Failed to open kernel.elf: {:?}", status);
    }
    let kernel_file = unsafe { &*memmap_file };

    // カーネル情報を取得
    let mut file_info_size: usize = size_of::<EfiFileInfo>();
    let file_info_buffer: *mut EfiFileInfo = null_mut::<EfiFileInfo>();
    output_writer.write_str("getting information for kernel.elf ...\n");
    let status = kernel_file.get_info(&EFI_FILE_INFO_GUID, &mut file_info_size, file_info_buffer);
    if status != EfiStatus::Success {
        output_writer.write_str("Failed to get information for kernel.elf\n");
        panic!(
            "Failed to get information for kernel.elf status: {:?}",
            status
        );
    }
    output_writer.write_str("success to get information for kernel.elf\n");
    let file_info_buffer = unsafe { &*file_info_buffer };
    let mut kernel_file_size: usize = file_info_buffer.file_size as usize;
    // writeln!(buf_writer, "file info: {file_info_buffer:?}");
    // output_writer.write_str(buf_writer.as_str().expect(""));

    // カーネルを展開する場所を確保する。
    let mut kernel_base_addr: u64 = 0x100000; // 展開する時の先頭アドレス
    let num_of_pages = (kernel_file_size + 0xfff) / 0x1000;
    let _ = writeln!(buf_writer, "number of pages for allocation: {num_of_pages}");
    output_writer.write_str(buf_writer.as_str().expect(""));
    buf_writer.flush();
    let status = efi_system_table.boot_services.allocate_pages(
        (kernel_file_size + 0xfff) / 0x1000,
        kernel_base_addr as *mut u64,
    );
    if status != EfiStatus::Success {
        output_writer.write_str("Failed to allocate page for kernel\n");
        panic!("Failed to allocate page for kernel {:?}", status);
    }
    output_writer.write_str("success allocate page\n");
    let status = kernel_file.read(
        &mut kernel_file_size,
        kernel_base_addr as usize as *mut EfiVoid, // kernel_base_addrを先頭にファイルの中身を展開する。
    );
    if status != EfiStatus::Success {
        output_writer.write_str("Failed to read to base addr\n");
        panic!("Failed to read to base addr {:?}", status);
    }
    output_writer.write_str("success deploy to base addr\n");

    // START: EFIのブートサービスを終了する
    let status = efi_system_table
        .boot_services
        .exit_boot_services(image_handle, memory_map.map_key);
    if status != EfiStatus::Success {
        let mut status = efi_system_table
            .boot_services
            .get_memory_map(&mut memory_map);
        if status != EfiStatus::Success {
            output_writer.write_str("failed to get memory map: ");
            output_writer.write_str(status.to_string());
            output_writer.write_str("\n");
            loop {}
        }
        status = efi_system_table
            .boot_services
            .exit_boot_services(image_handle, memory_map.map_key);
        if status != EfiStatus::Success {
            output_writer.write_str("Cloud not exit boot service: ");
            output_writer.write_str(status.to_string());
            output_writer.write_str("\n");
            loop {}
        }
    }
    // END

    // エントリーポイントを読み取る(エントリーポイント関数のアドレスは、elfヘッダーの24バイト後に書かれている).
    // let entry_addr: u64 = unsafe { *((kernel_base_addr + 24) as usize as *mut u64) };
    let entry_addr: u64 = (kernel_base_addr + 0x120);
    let header: [u8; 8] = unsafe { core::ptr::read((entry_addr) as *const u8 as *const [u8; 8]) };
    let _ = write!(buf_writer, "entry address: 0x{entry_addr:X} header: ");
    for h in header {
        let _ = write!(buf_writer, "{h:0>2X} ");
    }
    let _ = write!(buf_writer, "\n");
    output_writer.write_str(buf_writer.as_str().expect(""));
    buf_writer.flush();

    // エントリーアドレスを関数として実行する
    output_writer.write_str("execute kernel entry point\n");
    let entry_point: EntryPointType = unsafe { core::mem::transmute(entry_addr) };
    let entry_point_addr = entry_point as usize;
    let _ = writeln!(buf_writer, "{entry_point_addr:X}");
    output_writer.write_str(buf_writer.as_str().expect(""));
    buf_writer.flush();

    let panic_handler_addr = panic as usize;
    writeln!(
        buf_writer,
        "panic handler address: 0x{panic_handler_addr:X}"
    );
    output_writer.write_str(buf_writer.as_str().expect(""));
    buf_writer.flush();
    // entry_point();
    let kernel_res = entry_point();

    output_writer.write_str("ALL DONE.\n");
    loop {}
    EfiStatus::Success
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
