pub mod file;
pub mod graphics;
pub mod memory;
pub mod text;
pub mod types;

use core::marker::PhantomPinned;
use core::mem::offset_of;
use core::ptr::null_mut;

use crate::memory_map_holder::MemoryMapHolder;
use crate::uefi::memory::{EfiAllocateType, EfiMemoryType};
use graphics::*;
use text::EfiSimpleTextOutputProtocol;
use types::*;

// https://github.com/tianocore/edk2/blob/8216419a02173421ce7070268fdd11a7caadfa4b/MdePkg/Include/Uefi/UefiSpec.h#L2021
#[repr(C)]
pub struct EfiBootServicesTable {
    _reserved0: [u64; 5],
    allocate_pages: extern "win64" fn(
        allocate_type: EfiAllocateType,
        memory_type: EfiMemoryType,
        pages: usize,
        memory: *mut u64,
    ) -> EfiStatus,
    _reserved1: [u64; 1],
    get_memory_map: extern "win64" fn(
        // メモリマップを書き込む用のバッファのサイズを設定。小さすぎるとエラーとなる。
        memory_map_size: *mut usize,
        // メモリマップが書き込まれるバッファの先頭のポインタ
        memory_map: *mut u8,
        // メモリマップを識別するための値を書き込む変数
        // メモリマップは、UEFIの処理などで中身が変わってしまう。
        // この値が同じなら、メモリマップに変化がないことを示す。
        map_key: *mut usize,
        // メモリマップのここの行を表すメモリディスクリプタのバイト数
        descriptor_size: *mut usize,
        // メモリディスクリプタの構造体のバージョン番号を表す. not used in this implementation
        descriptor_version: *mut u32,
    ) -> EfiStatus,
    allocate_pool: extern "win64" fn(
        pool_type: EfiMemoryType,
        size: usize,
        buffer: *mut *mut EfiVoid,
    ) -> EfiStatus,
    free_pool: extern "win64" fn(buffer: *mut EfiVoid) -> EfiStatus,
    _reserved3: [u64; 19],
    exit_boot_services: extern "win64" fn(image_handle: EfiHandle, map_key: usize) -> EfiStatus,
    _reserved4: [u64; 5],
    // https://github.com/tianocore/edk2/blob/562bce0febd641f78df7cd61f2ed5a4c944b31ac/MdePkg/Include/Uefi/UefiSpec.h#L2021
    open_protocol: extern "win64" fn(
        handle: EfiHandle,
        protocol: *const EfiGuid,
        interface: *mut *mut EfiVoid,
        agent_handle: EfiHandle,
        controller_handle: EfiHandle,
        attributes: u32,
    ) -> EfiStatus,
    _reserved5: [u64; 3],
    locate_handle_buffer: extern "win64" fn(
        search_type: EfiLocateSearchType,
        protocol: *const EfiGuid,
        search_key: *const EfiVoid,
        no_handles: *mut usize,
        buffer: *mut *mut EfiHandle,
    ) -> EfiStatus,
    locate_protocol: extern "win64" fn(
        protocol: *const EfiGuid,
        registration: *const EfiVoid,
        interface: *mut *mut EfiVoid,
    ) -> EfiStatus,
}

impl EfiBootServicesTable {
    // EFI APIのメモリマップ取得APIからメモリマップを取得して、mapに格納する
    pub fn get_memory_map(&self, map: &mut MemoryMapHolder) -> EfiStatus {
        (self.get_memory_map)(
            &mut map.memory_map_size,
            map.memory_map_buffer.as_mut_ptr(), // memory_map_bufferはu8の配列なので、ポインタに変換して、先頭のポインタを渡す
            &mut map.map_key,
            &mut map.descriptor_size,
            &mut map.descriptor_version,
        )
    }

    pub fn allocate_pages(&self, pages: usize, memory: *mut u64) -> EfiStatus {
        (self.allocate_pages)(
            EfiAllocateType::AllocateAddress,
            EfiMemoryType::LOADER_DATA,
            pages,
            memory,
        )
    }

    pub fn allocate_pool(
        &self,
        pool_type: EfiMemoryType,
        size: usize,
        buffer: *mut *mut EfiVoid,
    ) -> EfiStatus {
        (self.allocate_pool)(pool_type, size, buffer)
    }
    pub fn free_pool(&self, buffer: *mut EfiVoid) -> EfiStatus {
        (self.free_pool)(buffer)
    }
    pub fn locate_handle_buffer(
        &self,
        search_key: EfiLocateSearchType,
        protocol: *const EfiGuid,
        no_handles: *mut usize,
        buffer: *mut *mut EfiHandle,
    ) -> EfiStatus {
        (self.locate_handle_buffer)(
            search_key,
            protocol,
            null_mut::<EfiVoid>(),
            no_handles,
            buffer,
        )
    }
    pub fn open_protocol(
        &self,
        handle: EfiHandle,
        protocol: *const EfiGuid,
        interface: *mut *mut EfiVoid,
        agent_handle: EfiHandle,
        controller_handle: EfiHandle,
    ) -> EfiStatus {
        (self.open_protocol)(
            handle,
            protocol,
            interface,
            agent_handle,
            controller_handle,
            EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL,
        )
    }

    pub fn exit_boot_services(&self, handle: EfiHandle, map_key: usize) -> EfiStatus {
        (self.exit_boot_services)(handle, map_key)
    }
}

// https://uefi.org/specs/UEFI/2.10/04_EFI_System_Table.html#id6
#[repr(C)]
pub struct EfiSystemTable {
    _header: [u64; 3],
    _firmware_vendor: EfiHandle,
    _firmware_revision: u32, // この後に、4バイトのパディングがある
    _reserved0: [u64; 3],
    pub con_out: &'static EfiSimpleTextOutputProtocol,
    _reserved1: [u64; 3],
    pub boot_services: &'static EfiBootServicesTable,
}

impl EfiSystemTable {
    pub fn con_out(&self) -> &'static EfiSimpleTextOutputProtocol {
        self.con_out
    }
}

const _: () = assert!(offset_of!(EfiSystemTable, boot_services) == 96);

// https://github.com/tianocore/edk2/blob/562bce0febd641f78df7cd61f2ed5a4c944b31ac/MdePkg/Include/Protocol/LoadedImage.h#L43
#[repr(C)]
#[derive(Debug)]
pub struct EfiLoadedImageProtocol {
    _padding0: [u64; 3],
    pub device_handle: EfiHandle,
    _pinned: PhantomPinned,
}
const _: () = assert!(offset_of!(EfiLoadedImageProtocol, device_handle) == 24);

// locate_protocolのオフセットを確認するためのアサーション(オフセットは、バイトで計算する)
const _: () = assert!(offset_of!(EfiBootServicesTable, get_memory_map) == 56);
const _: () = assert!(offset_of!(EfiBootServicesTable, locate_protocol) == 320);

pub fn open_gop<'a>(
    image_handle: EfiHandle,
    system_table: &EfiSystemTable,
) -> Result<&'a EfiGraphicsOutputProtocol<'a>> {
    let mut graphic_output_protocol = null_mut::<EfiGraphicsOutputProtocol>();
    let mut num_gop_handles: usize = 0;
    let mut gop_handles = null_mut::<EfiHandle>();

    let status = system_table.boot_services.locate_handle_buffer(
        EfiLocateSearchType::ByProtocol,
        &EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID,
        &mut num_gop_handles as *mut usize,
        &mut gop_handles as *mut *mut EfiHandle,
    );
    if status != EfiStatus::Success {
        return Err(Error::Failed("Failed to locate graphics output protocol"));
    }
    let gop_handles_array = unsafe { core::slice::from_raw_parts(gop_handles, num_gop_handles) };

    let status = system_table.boot_services.open_protocol(
        gop_handles_array[0],
        &EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID,
        &mut graphic_output_protocol as *mut *mut EfiGraphicsOutputProtocol as *mut *mut EfiVoid,
        image_handle,
        null_mut::<u64>() as u64,
    );
    if status != EfiStatus::Success {
        return Err(Error::Failed("Failed to open graphics output protocol"));
    }

    let _ = system_table.boot_services.free_pool(gop_handles as *mut u8);

    Ok(unsafe { &*graphic_output_protocol })
}
