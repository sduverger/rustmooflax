use rlibc::{memset, memcpy};
use core::slice;

// ELF Header
pub const EM_X86_64: u16 = 62;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Header {
    pub e_ident: [u8; 16],
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

// Program header
pub const PT_LOAD: u32 = 1;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

// Section Header
pub const SHT_RELA: u32 = 4;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SectionHeader {
    pub sh_name: u32,
    pub sh_type: u32,
    pub sh_flags: u64,
    pub sh_addr: u64,
    pub sh_offset: u64,
    pub sh_size: u64,
    pub sh_link: u32,
    pub sh_info: u32,
    pub sh_addralign: u64,
    pub sh_entsize: u64,
}

// Relocations
pub const R_X86_64_RELATIVE: u32 = 8;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Rela {
    pub r_offset: u64,
    pub r_info: u64,
    pub r_addend: i64,
}

impl Rela {
    #[inline(always)]
    pub fn r_type(&self) -> u32 { self.r_info as u32 }
}

pub trait ElfCast where Self: Sized {
    // #[inline(always)]
    // fn from_bytes(bytes: &[u8]) -> Option<&Self> {
    //     if bytes.len() < mem::size_of::<Self>() {
    //         None
    //     } else {
    //         Some(unsafe { & *(bytes.as_ptr() as *const _) })
    //     }
    // }

    #[inline(always)]
    fn from_addr<'a>(addr: u64) -> &'a Self {
        unsafe { & *(addr as *const u64 as *const _) }
    }

    #[inline(always)]
    fn as_addr(&self) -> u64 {
        (self as *const Self as _)
    }
}

impl ElfCast for Header {}
impl ElfCast for ProgramHeader {}
impl ElfCast for SectionHeader {}
impl ElfCast for Rela {}

impl Header {
    pub fn phdr(&self) -> Option<&ProgramHeader> {
        if self.e_phnum == 0 {
            return None
        }

        let addr = self.as_addr() + self.e_phoff;
        let phdr = ProgramHeader::from_addr(addr);

        if phdr.p_type == PT_LOAD {
            Some(phdr)
        } else {
            None
        }
    }

    pub fn shdr(&self, sht: u32) -> Option<&SectionHeader> {
        let ptr = (self.as_addr() + self.e_shoff) as *const SectionHeader;
        let shdrs = unsafe {
            slice::from_raw_parts(ptr, self.e_shnum as usize)
        };

        for shdr in shdrs.iter() {
            if shdr.sh_type == sht {
                return Some(shdr)
            }
        }

        None
    }
}

// Public API

pub fn module_load(addr: u64, base: u64) {
    let ehdr = Header::from_addr(addr);

    if ehdr.e_machine != EM_X86_64 {
        panic!("invalid elf module");
    }

    let phdr = match ehdr.phdr() {
        None => panic!("invalid program header"),
        Some(p) => p,
    };

    let file = addr + phdr.p_offset;
    let mut mem = base + phdr.p_vaddr;
    unsafe { memcpy(mem as *mut u8, file as *mut u8, phdr.p_filesz as usize); }

    let pad = phdr.p_memsz - phdr.p_filesz;
    if pad > 0 {
        mem += phdr.p_filesz;
        unsafe { memset(mem as *mut u8, 0, pad as usize); }
    }

    if let Some(rhdr) = ehdr.shdr(SHT_RELA) {
        let rel_ptr = (addr + rhdr.sh_offset) as *const Rela;
        let rel_nr  = (rhdr.sh_size / rhdr.sh_entsize) as usize;
        let relocs = unsafe {
            slice::from_raw_parts(rel_ptr, rel_nr)
        };

        for rela in relocs.iter() {
            if rela.r_type() != R_X86_64_RELATIVE {
                panic!("invalid relocation");
            }

            if rela.r_addend < 0 {
                panic!("invalid relocation addend");
            }

            let fix = (base + rela.r_offset) as *mut u64;
            unsafe { *fix = base + rela.r_addend as u64; }
        }
    }
}

pub fn module_entry(addr: u64) -> u64 {
    Header::from_addr(addr).e_entry
}

pub fn module_size(addr: u64) -> usize {
    let ehdr = Header::from_addr(addr);
    if let Some(phdr) = ehdr.phdr() {
        phdr.p_memsz as usize
    } else {
        panic!("Invalid Program header")
    }
}

// 0 or 1 means no alignment,
// force 16 bytes 
pub fn module_align(addr: u64) -> usize {
    let ehdr = Header::from_addr(addr);
    if let Some(phdr) = ehdr.phdr() {
        if phdr.p_align < 2 {
            16
        } else {
            phdr.p_align as usize
        }
    } else {
        panic!("Invalid Program header")
    }
}
