// Real mode
use core::slice;
use core::mem;

// Memory map
pub const IVT_START             : u64 = 0x000000; // 1KB IVT
pub const IVT_END               : u64 = 0x000400;
pub const BIOS_DATA_START       : u64 = 0x000400; // 512B BIOS data area
pub const BIOS_DATA_END         : u64 = 0x000600;
pub const LOW_MEM_START         : u64 = 0x000600; // conventional memory
pub const LOW_MEM_END           : u64 = 0x09fc00;
pub const BIOS_EXT_DATA_START   : u64 = 0x09fc00; // BIOS extended data area
pub const BIOS_EXT_DATA_END     : u64 = 0x0a0000;
pub const VGA_START             : u64 = 0x0a0000; // 128KB of VGA memory
pub const VGA_END               : u64 = 0x0c0000; // 192KB VGA BIOS rom
pub const VGA_BIOS_START        : u64 = 0x0c0000;
pub const VGA_BIOS_END          : u64 = 0x0f0000;
pub const BIOS_START            : u64 = 0x0f0000; // 64KB BIOS rom
pub const BIOS_START_OFFSET     : u64 = 0x00fff0;
pub const BIOS_END              : u64 = 0x100000;
pub const EXT_MEM_START         : u64 = (1<<20);  // Extended memory
pub const DMA_END               : u64 = (16<<20); // DMA limit
pub const BIOS_BOOTLOADER       : u64 = 0x07c00;  // BIOS bootloader @

// rmode max addr is 0xffff<<4 + 0xffff == 0x10ffef == WRAP_LIMIT
//
// - if a20 is off, we emulate wrap-around by mirroring
//   pages from [ 1MB ; 1MB+64KB [ to [ 0 ; 64KB [
//
// - if a20 is on, we do not wrap-around
pub const LIMIT             : u64 = EXT_MEM_START;
pub const WRAP_LIMIT        : u64 = (LIMIT + (64<<10));
pub const STACK_BOTTOM      : u64 = ((LOW_MEM_END & 0xffff0000) - 2);
pub const BASE_SS           : u64 = ((STACK_BOTTOM & 0xffff0000)>>4);
pub const BASE_SP           : u64 = (STACK_BOTTOM & 0xffff);
pub const BASE_IP           : u64 = (LOW_MEM_START & 0xffff);
pub const INT19             : u16 = 0x19cd;

// Interrupt Vector Table entry
#[derive(Debug,Default,Copy,Clone)]
pub struct IVTEntry {
    pub ip: u16,
    pub cs: u16,
}

impl IVTEntry {
    pub fn as_mut_u8(&mut self) -> &mut[u8] {
        let ptr = self as *const _ as *mut u8;
        unsafe { slice::from_raw_parts_mut(ptr, mem::size_of::<IVTEntry>()) }
    }
}

pub const BIOS_VIDEO_INTERRUPT          : u8 = 0x10;
pub const BIOS_DISK_INTERRUPT           : u8 = 0x13;
pub const BIOS_MISC_INTERRUPT           : u8 = 0x15;
pub const BIOS_KBD_INTERRUPT            : u8 = 0x16;
pub const BIOS_BOOT_INTERRUPT           : u8 = 0x19;

// BIOS services related to MISC_INTERRUPT

// AX values
pub const BIOS_GET_SMAP                 : u16 = 0xe820;
pub const BIOS_SMAP_ID                  : u16 = 0x534d4150;
pub const BIOS_SMAP_ERROR               : u16 = 0x86;
pub const BIOS_GET_EXT_MEM_32           : u16 = 0xe881;
pub const BIOS_GET_EXT_MEM              : u16 = 0xe801;
pub const BIOS_DISABLE_A20              : u16 = 0x2400;
pub const BIOS_ENABLE_A20               : u16 = 0x2401;
pub const BIOS_STATUS_A20               : u16 = 0x2402;
pub const BIOS_SUPPORT_A20              : u16 = 0x2403;

// AH values
pub const BIOS_GET_BIG_MEM              : u8 = 0x8a;
pub const BIOS_OLD_GET_EXT_MEM          : u8 = 0x88;


// Some services

pub fn is_bios_mem(addr: u64) -> bool {
    (addr >= IVT_START && addr < BIOS_DATA_END) ||
        (addr >= BIOS_EXT_DATA_START && addr < EXT_MEM_START)
}

pub fn ivt_limit(n: u8) -> u16 {
    ((n as usize * mem::size_of::<IVTEntry>()) - 1) as u16
}

pub fn vm_set_entry(addr: u64) {
    let entry = addr as *mut u16;
    unsafe { *entry = INT19; }
}
