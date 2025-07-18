#![no_std]
#![no_main]

use core::arch::asm;
use core::mem::size_of;
use core::panic::PanicInfo;
use core::slice;
pub mod memory_map_holder;
pub mod uefi;
// mod uefi_alloc;

use memory_map_holder::MemoryMapHolder;
use uefi::types::{EfiHandle, EfiStatus};
use uefi::*;

const MEMMAP_PATH: &[u16] = &[
    b'\\' as u16,
    b'\\' as u16,
    b'm' as u16,
    b'e' as u16,
    b'm' as u16,
    b'm' as u16,
    b'a' as u16,
    b'p' as u16,
    0u16, // NULL終端
];

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

    output_writer.write_str("Successfully got memory map\n");
    loop {
        // unsafe {
        //     asm!("hlt");
        // }
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
