use utils::{align, align_next, aligned};

// Paging utilities

pub const PG_32_SHIFT:   usize = 5;
pub const PG_4K_SHIFT:   usize = 12;
pub const PG_2M_SHIFT:   usize = 21;
pub const PG_4M_SHIFT:   usize = 22;
pub const PG_1G_SHIFT:   usize = 30;
pub const PG_512G_SHIFT: usize = 39;

pub const fn pg_size(shift: usize) -> usize { 1 << shift }

pub fn pg_offset(shift: usize, addr: u64) -> u64 {
    addr & ((pg_size(shift) - 1) as u64)
}

pub fn pg_nr(shift: usize, addr: u64) -> usize {
    (addr as usize) >> shift
}

pub fn pg_addr(shift: usize, bits: u64) -> u64 {
    bits << (shift as u64)
}

pub fn pg_align(shift: usize, addr: u64) -> u64 {
    align(addr, pg_size(shift))
}

pub fn pg_align_next(shift: usize, addr: u64) -> u64 {
    align_next(addr, pg_size(shift))
}

pub fn pg_aligned(shift: usize, addr: u64) -> bool {
    aligned(addr, pg_size(shift))
}


// convenient aliases
pub const PG_4KB:   usize = pg_size(PG_4K_SHIFT);
pub const PG_32B:   usize = pg_size(PG_32_SHIFT);
pub const PG_2MB:   usize = pg_size(PG_2M_SHIFT);
pub const PG_4MB:   usize = pg_size(PG_4M_SHIFT);
pub const PG_1GB:   usize = pg_size(PG_1G_SHIFT);
pub const PG_512GB: usize = pg_size(PG_512G_SHIFT);

pub const PML4E_PER_PML4:   usize = 512;
pub const PDPE_PER_PDP:     usize = 512;
pub const PD64E_PER_PD64:   usize = 512;
pub const PT64E_PER_PT64:   usize = 512;
pub const PD32E_PER_PD32:   usize = 1024;
pub const PT32E_PER_PT32:   usize = 1024;

pub const PML4_SZ: usize = PML4E_PER_PML4 * 8;
pub const PDP_SZ:  usize = PDPE_PER_PDP   * 8;
pub const PD64_SZ: usize = PD64E_PER_PD64 * 8;
pub const PT64_SZ: usize = PT64E_PER_PT64 * 8;

pub fn pfn(addr: u64) -> usize { pg_nr(PG_4K_SHIFT, addr) }


// relative index for paging walk through
pub fn pml4_idx(addr: u64) -> usize {
    ((addr as usize) >> PG_512G_SHIFT) & 0x1ff
}

pub fn pdp_idx(addr:  u64) -> usize {
    ((addr as usize) >> PG_1G_SHIFT) & 0x1ff
}

pub fn pdp_pae_idx(addr: u64) -> usize {
    ((addr as usize) >> PG_1G_SHIFT) & 0x3
}

pub fn pd64_idx(addr: u64) -> usize {
    ((addr as usize) >> PG_2M_SHIFT) & 0x1ff
}

pub fn pt64_idx(addr: u64) -> usize {
    ((addr as usize) >> PG_4K_SHIFT) & 0x1ff
}

pub fn pd32_idx(addr: u64) -> usize {
    ((addr as usize) >> PG_4M_SHIFT) & 0x3ff
}

pub fn pt32_idx(addr: u64) -> usize {
    ((addr as usize) >> PG_4K_SHIFT) & 0x3ff
}

// absolute index (some give table number)
pub fn pg_abs_idx(shift: usize, addr: u64) -> usize {
    (addr as usize) >> shift
}

pub fn pdp_nr(addr: u64)  -> usize {
    (addr as usize) >> PG_512G_SHIFT
}

pub fn pd64_nr(addr: u64) -> usize {
    (addr as usize) >> PG_1G_SHIFT  
}

pub fn pt64_nr(addr: u64) -> usize {
    (addr as usize) >> PG_2M_SHIFT  
}

// PageEntry bits
pub const PG_P:         u64 = 1;
pub const PG_RO:        u64 = 0;
pub const PG_RW:        u64 = 1<<1;
pub const PG_KRN:       u64 = 0;
pub const PG_USR:       u64 = 1<<2;
pub const PG_PWT:       u64 = 1<<3;
pub const PG_PCD:       u64 = 1<<4;
pub const PG_ACC:       u64 = 1<<5;
pub const PG_DRT:       u64 = 1<<6;
pub const PG_PAT:       u64 = 1<<7;
pub const PG_GLB:       u64 = 1<<8;
pub const PG_NX:        u64 = 1<<63;

// Large pages
pub const PG_PS:        u64 = 1<<7;
pub const PG_LPAT:      u64 = 1<<12;

// Page Table address extraction bitmask
pub fn addr_mask(shift: usize) -> u64 {
    let len = 52 - shift;
    (((1<<len) - 1) << shift)
}
