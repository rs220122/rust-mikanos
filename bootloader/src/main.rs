#![no_std]
#![no_main]

use core::arch::asm;

use core::mem::size_of;
use core::panic::PanicInfo;
use core::ptr::null_mut;
use core::slice;
pub mod memory_map_holder;
pub mod uefi;
// mod uefi_alloc;

use memory_map_holder::MemoryMapHolder;
use uefi::text::EfiSimpleTextOutputProtocolWriter;
use uefi::types::{
    EFI_FILE_MODE_CREATE, EFI_FILE_MODE_READ, EFI_FILE_MODE_WRITE, EFI_LOADED_IMAGE_PROTOCOL_GUID,
    EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_GUID, EfiHandle, EfiStatus, EfiVoid,
};
use uefi::*;

const memmap_path: &[u16; 12] = &[
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
    let fs = unsafe { &*fs };

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
    memmap_file: *mut *mut EfiFileProtocol,
) -> EfiStatus {
    let mut output_writer = EfiSimpleTextOutputProtocolWriter::new(efi_system_table.con_out());

    let status = root.open(
        memmap_file,
        memmap_path,
        EFI_FILE_MODE_READ | EFI_FILE_MODE_WRITE | EFI_FILE_MODE_CREATE,
        0,
    );

    if status != EfiStatus::Success {
        output_writer.write_str("Failed to open memory map file");
        return status;
    }
    output_writer.write_str("Successfully opened memory map file status: ");
    output_writer.write_str(status.to_string());
    output_writer.write_str("\n");

    return status;
}

#[unsafe(no_mangle)]
/// The entry point of the bootloader
pub extern "C" fn efi_main(
    image_handle: EfiHandle,
    efi_system_table: &EfiSystemTable,
) -> EfiStatus {
    let efi_graphics_output_protocol = locate_graphic_protocol(efi_system_table).unwrap();
    let vram_addr: usize = efi_graphics_output_protocol.mode.frame_buffer_base;
    let vram_byte_size: usize = efi_graphics_output_protocol.mode.frame_buffer_size;
    // フレームバッファの配列を獲得する。フレームバッファの一つのピクセルは、u32で表現される。
    // そのため、vram_addrをu32のポインタにキャスト
    let vram = unsafe {
        slice::from_raw_parts_mut(vram_addr as *mut u32, vram_byte_size / size_of::<u32>())
    };
    // for e in vram {
    //     *e = 0xffffff;
    // }

    // con_outで出力
    let con_out = efi_system_table.con_out();
    let mut output_writer = EfiSimpleTextOutputProtocolWriter::new(con_out);

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

    // memmapファイルを閉じる
    let status = memmap_file.close();
    if status != EfiStatus::Success {
        output_writer.write_str("Failed to close memory map file");
        panic!("Failed to close memory map file: {:?}", status);
    }
    output_writer.write_str("Successfully closed memory map file status: ");
    output_writer.write_str(status.to_string());
    output_writer.write_str("\n");

    loop {
        unsafe {
            asm!("hlt");
        }
    }
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
