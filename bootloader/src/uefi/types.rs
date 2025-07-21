use core::fmt;

pub type EfiVoid = u8;
pub type EfiHandle = u64;
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    EfiError(EfiStatus),
    Failed(&'static str),
}

impl From<EfiStatus> for Error {
    fn from(e: EfiStatus) -> Self {
        Error::EfiError(e)
    }
}
pub type Result<T> = core::result::Result<T, Error>;

// https://github.com/tianocore/edk2/blob/562bce0febd641f78df7cd61f2ed5a4c944b31ac/MdePkg/Include/Uefi/UefiSpec.h#L1351C9-L1351C58
pub const EFI_OPEN_PROTOCOL_BY_HANDLE_PROTOCOL: u32 = 00000001; // OpenProtocolの属性

pub const EFI_FILE_MODE_READ: u64 = 0x0000000000000001;
pub const EFI_FILE_MODE_WRITE: u64 = 0x0000000000000002;
pub const EFI_FILE_MODE_CREATE: u64 = 0x8000000000000000;

// protocol GUIDs
pub const EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID: EfiGuid = EfiGuid {
    data0: 0x9042a9de,
    data1: 0x23dc,
    data2: 0x4a38,
    data3: [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a],
};
pub const EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_GUID: EfiGuid = EfiGuid {
    data0: 0x964e5b22,
    data1: 0x6459,
    data2: 0x11d2,
    data3: [0x8e, 0x39, 0x0, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
};
pub const EFI_LOADED_IMAGE_PROTOCOL_GUID: EfiGuid = EfiGuid {
    data0: 0x5b1b31a1,
    data1: 0x9562,
    data2: 0x11d2,
    data3: [0x8e, 0x3f, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
};
pub const EFI_FILE_INFO_GUID: EfiGuid = EfiGuid {
    data0: 0x9576e92,
    data1: 0x6d3f,
    data2: 0x11d2,
    data3: [0x8e, 0x39, 0x0, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
};

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EfiGuid {
    pub data0: u32,
    pub data1: u16,
    pub data2: u16,
    pub data3: [u8; 8],
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[must_use]
#[repr(u64)]
pub enum EfiStatus {
    Success = 0,
}
impl EfiStatus {
    pub fn into_result(self) -> Result<()> {
        if self == EfiStatus::Success {
            Ok(())
        } else {
            Err(self.into())
        }
    }

    pub fn to_string(self) -> &'static str {
        match self {
            EfiStatus::Success => "Success",
            _ => "Unknown status",
        }
    }
}

#[repr(C)]
#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct EfiTime {
    year: u16,  // 1900 – 9999
    month: u8,  // 1 – 12
    day: u8,    // 1 – 31
    hour: u8,   // 0 – 23
    minute: u8, // 0 – 59
    second: u8, // 0 – 59
    pad1: u8,
    nanosecond: u32, // 0 – 999,999,999
    time_zone: u16,  // -1440 to 1440 or 2047
    daylight: u8,
    pad2: u8,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(i32)]
pub enum EfiLocateSearchType {
    AllHandles = 0,
    ByRegisterNotify,
    ByProtocol,
}
