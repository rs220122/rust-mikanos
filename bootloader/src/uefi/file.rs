use crate::uefi::types::{EfiGuid, EfiStatus, EfiTime, EfiVoid};
use core::mem::offset_of;
#[repr(C)]
#[derive(Default, Debug)]
pub struct EfiFileName {
    name: [u16; 12],
}

#[repr(C)]
#[derive(Default, Debug)]
pub struct EfiFileInfo {
    size: u64,
    pub file_size: u64,
    pub physical_size: u64,
    pub create_time: EfiTime,
    pub last_access_time: EfiTime,
    pub modification_time: EfiTime,
    pub attr: u64,
    pub file_name: EfiFileName,
}

// GUID SIMPLE FILE SYSTEM PROTOCOLの実装
#[repr(C)]
#[derive(Debug)]
pub struct EfiSimpleFileSystemProtocol {
    pub revision: u64,
    pub open_volume:
        extern "win64" fn(this: *const Self, root: *mut *mut EfiFileProtocol) -> EfiStatus,
}

impl EfiSimpleFileSystemProtocol {
    pub fn open_volume(&self, root: *mut *mut EfiFileProtocol) -> EfiStatus {
        (self.open_volume)(self as *const Self, root)
    }
}

// https://github.com/tianocore/edk2/blob/master/MdePkg/Include/Protocol/SimpleFileSystem.h#L528
#[repr(C)]
#[derive(Debug)]
pub struct EfiFileProtocol {
    pub revision: u64,
    pub open: extern "win64" fn(
        this: *const Self,
        new_handle: *mut *mut Self,
        file_name: *const u16,
        open_mode: u64,
        attributes: u64,
    ) -> EfiStatus,
    pub close: extern "win64" fn(this: *mut EfiFileProtocol) -> EfiStatus,
    _reserved0: [u64; 1],
    pub read: extern "win64" fn(
        this: *const EfiFileProtocol,
        buffer_size: *mut usize,
        buffer: *mut EfiVoid,
    ) -> EfiStatus,
    pub write: extern "win64" fn(
        this: *mut EfiFileProtocol,
        buffer_size: *mut usize,
        buffer: *mut EfiVoid,
    ) -> EfiStatus,
    _reserved1: [u64; 2],
    pub get_info: extern "win64" fn(
        this: *const Self,
        information_type: *const EfiGuid,
        buffer_size: *mut usize,
        buffer: *mut EfiVoid,
    ) -> EfiStatus,
    _reserved2: [u64; 6],
}
impl EfiFileProtocol {
    pub fn open(
        &self,
        new_handle: &mut *mut EfiFileProtocol,
        file_name: &[u16],
        open_mode: u64,
        attributes: u64,
    ) -> EfiStatus {
        (self.open)(
            self as *const _ as *mut EfiFileProtocol,
            new_handle as *mut *mut EfiFileProtocol,
            file_name.as_ptr(),
            open_mode,
            attributes,
        )
    }

    pub fn read(&self, buffer_size: &mut usize, buffer: *mut EfiVoid) -> EfiStatus {
        (self.read)(self as *const Self, buffer_size as *mut usize, buffer)
    }
    pub fn close(&self) -> EfiStatus {
        (self.close)(self as *const _ as *mut EfiFileProtocol)
    }
    pub fn get_info(
        &self,
        information_type: &EfiGuid,
        buffer_size: &mut usize,
        buffer: *mut EfiFileInfo,
    ) -> EfiStatus {
        (self.get_info)(
            self as *const _ as *mut EfiFileProtocol,
            information_type as *const EfiGuid,
            buffer_size as *mut usize,
            buffer as *mut EfiVoid,
        )
    }
    pub fn write_char(&self, c: u8) -> EfiStatus {
        (self.write)(
            self as *const _ as *mut EfiFileProtocol,
            &mut 1,                     // 書き込むバイト数
            &c as *const u8 as *mut u8, // 書き込むバッファのポインタ
        )
    }

    pub fn write_str(&self, s: &str) -> EfiStatus {
        for c in s.bytes() {
            if c == b'\n' {
                let status = self.write_char(b'\r');
                if status != EfiStatus::Success {
                    return status; // 改行文字の書き込みに失敗した場合は、エラーを返す
                }
            }
            let status = self.write_char(c);
            if status != EfiStatus::Success {
                return status;
            }
        }
        EfiStatus::Success
    }
}

const _: () = assert!(offset_of!(EfiSimpleFileSystemProtocol, open_volume) == 8);
const _: () = assert!(offset_of!(EfiFileProtocol, open) == 8);
const _: () = assert!(offset_of!(EfiFileProtocol, close) == 16);
const _: () = assert!(offset_of!(EfiFileProtocol, write) == 40);
