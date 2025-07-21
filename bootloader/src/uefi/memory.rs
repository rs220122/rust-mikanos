#[repr(i64)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EfiMemoryType {
    RESERVED = 0,
    LOADER_CODE,
    LOADER_DATA,
    BOOT_SERVICES_CODE,
    BOOT_SERVICES_DATA,
    RUNTIME_SERVICES_CODE,
    RUNTIME_SERVICES_DATA,
    CONVENTIONAL_MEMORY,
    UNUSABLE_MEMORY,
    ACPI_RECLAIM_MEMORY,
    ACPI_MEMORY_NVS,
    MEMORY_MAPPED_IO,
    MEMORY_MAPPED_IO_PORT_SPACE,
    PAL_CODE,
    PERSISTENT_MEMORY,
}

#[repr(usize)]
pub enum EfiAllocateType {
    AllocateAnyPages = 0,
    AllocateMaxAddress,
    AllocateAddress,
    MaxAllocateType,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EfiMemoryDescriptor {
    pub memory_type: EfiMemoryType, // メモリ領域の種別
    pub physical_start: u64,        // メモリ領域先頭の物理メモリアドレス
    pub virtual_start: u64,         // メモリ領域先頭の仮想メモリアドレス
    pub number_of_pages: u64,       // メモリ領域の大きさ(4KiBページ単位)
    pub attribute: u64,             // メモリ領域が使える用途を示すビット集合
}

impl EfiMemoryDescriptor {
    pub fn get_memory_type_str(&self) -> &str {
        match self.memory_type {
            EfiMemoryType::RESERVED => "RESERVED",
            EfiMemoryType::LOADER_CODE => "LOADER_CODE",
            EfiMemoryType::LOADER_DATA => "LOADER_DATA",
            EfiMemoryType::BOOT_SERVICES_CODE => "BOOT_SERVICES_CODE",
            EfiMemoryType::BOOT_SERVICES_DATA => "BOOT_SERVICES_DATA",
            EfiMemoryType::RUNTIME_SERVICES_CODE => "RUNTIME_SERVICES_CODE",
            EfiMemoryType::RUNTIME_SERVICES_DATA => "RUNTIME_SERVICES_DATA",
            EfiMemoryType::CONVENTIONAL_MEMORY => "CONVENTIONAL_MEMORY",
            EfiMemoryType::UNUSABLE_MEMORY => "UNUSABLE_MEMORY",
            EfiMemoryType::ACPI_RECLAIM_MEMORY => "ACPI_RECLAIM_MEMORY",
            EfiMemoryType::ACPI_MEMORY_NVS => "ACPI_MEMORY_NVS",
            EfiMemoryType::MEMORY_MAPPED_IO => "MEMORY_MAPPED_IO",
            EfiMemoryType::MEMORY_MAPPED_IO_PORT_SPACE => "MEMORY_MAPPED_IO_PORT_SPACE",
            EfiMemoryType::PAL_CODE => "PAL_CODE",
            EfiMemoryType::PERSISTENT_MEMORY => "PERSISTENT_MEMORY",
        }
    }
}
