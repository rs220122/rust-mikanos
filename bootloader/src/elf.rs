// ELFヘッダーの内容を表現する構造体を定義
const EI_NIDENT: usize = 16;

// Elf File Header
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ElfEhdr {
    pub e_ident: [u8; EI_NIDENT],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

//Elf Program Header
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ElfPhdr {
    pub p_type: ElfPhdrType,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ElfPhdrType {
    PtNull = 0,
    PtLoad,
    PtDynamic,
    PtInterp,
    PtNote,
    PtShlib,
    PtPhdr,
    PtTls,
}
