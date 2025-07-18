use core::fmt;
use core::marker::PhantomPinned;
use core::mem::offset_of;
use core::ptr::null_mut;

pub mod graphics;
pub mod memory;
pub mod types;
use crate::memory_map_holder::MemoryMapHolder;
use graphics::*;
use types::*;

#[repr(C)]
pub struct EfiBootServicesTable {
    _reserved0: [u64; 7],
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
    _reserved1: [u64; 27],
    // https://github.com/tianocore/edk2/blob/562bce0febd641f78df7cd61f2ed5a4c944b31ac/MdePkg/Include/Uefi/UefiSpec.h#L2021
    open_protocol: extern "win64" fn(
        handle: EfiHandle,
        protocol: *const EfiGuid,
        interface: *mut *mut EfiVoid,
        agent_handle: EfiHandle,
        controller_handle: EfiHandle,
        attributes: u32,
    ) -> EfiStatus,
    _reserved2: [u64; 4],
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
}

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

#[repr(C)]
pub struct EfiSimpleTextOutputProtocol {
    reset: EfiHandle,
    output_string:
        extern "win64" fn(this: *const EfiSimpleTextOutputProtocol, str: *const u16) -> EfiStatus,
    test_string: EfiHandle,
    query_mode: EfiHandle,
    set_mode: EfiHandle,
    set_attribute: EfiHandle,
    clear_screen: extern "win64" fn(this: *const EfiSimpleTextOutputProtocol) -> EfiStatus,
    //
    _pinned: PhantomPinned,
}

impl EfiSimpleTextOutputProtocol {
    pub fn clear_screen(&self) -> Result<()> {
        (self.clear_screen)(self).into_result()
    }
}

// https://github.com/tianocore/edk2/blob/562bce0febd641f78df7cd61f2ed5a4c944b31ac/MdePkg/Include/Protocol/LoadedImage.h#L43
#[repr(C)]
#[derive(Debug)]
pub struct EfiLoadedImageProtocol {
    _padding0: [u64; 3],
    pub device_handle: EfiHandle,
    _pinned: PhantomPinned,
}
const _: () = assert!(offset_of!(EfiLoadedImageProtocol, device_handle) == 24);

pub struct EfiSimpleTextOutputProtocolWriter<'a> {
    pub protocol: &'a EfiSimpleTextOutputProtocol,
    //
    _pinned: PhantomPinned,
}

impl<'a> EfiSimpleTextOutputProtocolWriter<'a> {
    pub fn new(protocol: &'a EfiSimpleTextOutputProtocol) -> Self {
        EfiSimpleTextOutputProtocolWriter {
            protocol,
            _pinned: PhantomPinned,
        }
    }
}

impl EfiSimpleTextOutputProtocolWriter<'_> {
    pub fn write_char(&mut self, c: u8) {
        let cbuf: [u16; 2] = [c.into(), 0];
        (self.protocol.output_string)(self.protocol, cbuf.as_ptr())
            .into_result()
            .unwrap();
    }
    pub fn write_str(&mut self, s: &str) {
        for c in s.bytes() {
            if c == b'\n' {
                self.write_char(b'\r');
            }
            self.write_char(c);
        }
    }
}

impl fmt::Write for EfiSimpleTextOutputProtocolWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

// GUID SIMPLE FILE SYSTEM PROTOCOLの実装
#[repr(C)]
#[derive(Debug)]
pub struct EfiSimpleFileSystemProtocol {
    pub revision: u64,
    pub open_volume: extern "win64" fn(
        this: *mut EfiSimpleFileSystemProtocol,
        root: *mut *mut EfiFileProtocol,
    ) -> EfiStatus,
}

#[repr(C)]
#[derive(Debug)]
pub struct EfiFileProtocol {
    pub revision: u64,
    pub open: extern "win64" fn(
        this: *mut EfiFileProtocol,
        new_handle: *mut *mut EfiFileProtocol,
        file_name: *const u16,
        open_mode: u64,
        attributes: u64,
    ) -> EfiStatus,
    pub close: extern "win64" fn(this: *mut EfiFileProtocol) -> EfiStatus,
    _reserved0: [u64; 2],
    pub write: extern "win64" fn(
        this: *mut EfiFileProtocol,
        buffer_size: *mut usize,
        buffer: *mut u8,
    ) -> EfiStatus,
    _reserved1: [u64; 9],
}

// locate_protocolのオフセットを確認するためのアサーション(オフセットは、バイトで計算する)
const _: () = assert!(offset_of!(EfiBootServicesTable, get_memory_map) == 56);
const _: () = assert!(offset_of!(EfiBootServicesTable, locate_protocol) == 320);
const _: () = assert!(offset_of!(EfiSimpleFileSystemProtocol, open_volume) == 8);
const _: () = assert!(offset_of!(EfiFileProtocol, open) == 8);
const _: () = assert!(offset_of!(EfiFileProtocol, close) == 16);
const _: () = assert!(offset_of!(EfiFileProtocol, write) == 40);

pub fn locate_graphic_protocol<'a>(
    efi_system_table: &EfiSystemTable,
) -> Result<&'a EfiGraphicsOutputProtocol<'a>> {
    // 入れるためのポインタをnull_mutで初期化
    let mut graphic_output_protocol = null_mut::<EfiGraphicsOutputProtocol>();
    let status = (efi_system_table.boot_services.locate_protocol)(
        &EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID,
        null_mut::<EfiVoid>(), // 登録はしないのでnull_mut
        &mut graphic_output_protocol as *mut *mut EfiGraphicsOutputProtocol as *mut *mut EfiVoid,
    );
    if status != EfiStatus::Success {
        return Err(Error::Failed("Failed to locate graphics output protocol"));
    }
    Ok(unsafe { &*graphic_output_protocol })
}

pub fn open_root_dir(
    image_handle: EfiHandle,
    root: &mut *mut EfiFileProtocol,
    efi_system_table: &EfiSystemTable,
) -> EfiStatus {
    let mut loaded_image: *mut EfiLoadedImageProtocol = null_mut::<EfiLoadedImageProtocol>();
    let mut fs: *mut EfiSimpleFileSystemProtocol = null_mut::<EfiSimpleFileSystemProtocol>();

    // OpenProtocolを呼び出して、loaded_imageとfsを取得
    let status = ((efi_system_table.boot_services.open_protocol)(
        image_handle,
        &EFI_LOADED_IMAGE_PROTOCOL_GUID,
        &mut loaded_image as *mut *mut EfiLoadedImageProtocol as *mut *mut EfiVoid,
        image_handle,
        null_mut::<u64>() as u64,
        EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL,
    ));
    if status != EfiStatus::Success {
        return status;
    }
    let status = ((efi_system_table.boot_services.open_protocol)(
        loaded_image as u64,
        &EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_GUID,
        &mut fs as *mut *mut EfiSimpleFileSystemProtocol as *mut *mut EfiVoid,
        image_handle,
        null_mut::<u64>() as u64,
        EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL,
    ));
    if status != EfiStatus::Success {
        return status;
    }

    // open_volumeを呼び出して、ルートディレクトリを取得
    let status = unsafe { ((*fs).open_volume)(fs, root as *mut *mut EfiFileProtocol) };

    status
}
