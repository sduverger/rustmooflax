// x86-64 4 level paging
use paging::utils::*;

#[derive(Debug,Copy,Clone)]
pub enum PagingLevel {
    L4,
    L3,
    L2,
    L1,
}

pub fn pg_level_shift(lv: PagingLevel) -> usize {
    match lv {
        PagingLevel::L4 => PG_512G_SHIFT,
        PagingLevel::L3 => PG_1G_SHIFT,
        PagingLevel::L2 => PG_2M_SHIFT,
        PagingLevel::L1 => PG_4K_SHIFT,
    }
}

pub fn pg_level_size(lv: PagingLevel) -> usize {
    match lv {
        PagingLevel::L4 => 1<<PG_512G_SHIFT,
        PagingLevel::L3 => 1<<PG_1G_SHIFT,
        PagingLevel::L2 => 1<<PG_2M_SHIFT,
        PagingLevel::L1 => 1<<PG_4K_SHIFT,
    }
}

pub fn pg_level_next(lv: PagingLevel) -> Option<PagingLevel> {
    match lv {
        PagingLevel::L4 => Some(PagingLevel::L3),
        PagingLevel::L3 => Some(PagingLevel::L2),
        PagingLevel::L2 => Some(PagingLevel::L1),
        PagingLevel::L1 => None,
    }
}

pub fn pg_level_prev(lv: PagingLevel) -> Option<PagingLevel> {
    match lv {
        PagingLevel::L4 => None,
        PagingLevel::L3 => Some(PagingLevel::L4),
        PagingLevel::L2 => Some(PagingLevel::L3),
        PagingLevel::L1 => Some(PagingLevel::L2),
    }
}


// Paging Environment
pub struct PagingEnv<'a,T:'a> {
    pub root: &'a mut T,
    pub asid: u16,
}

impl<'a,T> PagingEnv<'a, T> {
    pub fn get_addr(&self) -> u64 {
        self.root as *const _ as u64
    }
}


// Helper for pre-computing ptb memory size
pub struct PagingSize {
    pub pdp: usize,
    pub pd: usize,
    pub pt: usize,
}

impl PagingSize {
    pub fn size(&self) -> usize {
        (PML4_SZ + self.pdp*PDP_SZ + self.pd*PD64_SZ + self.pt*PT64_SZ)
    }
}



// Page Entry Operations
pub const PG_OP_ADDR: u64 = 1<<0;  // update addr field of table entries
pub const PG_OP_OFF:  u64 = 1<<1;  // add offset to addr field
pub const PG_OP_PVL:  u64 = 1<<2;  // update pvl part of table entries
pub const PG_OP_CCH:  u64 = 1<<3;  // update cache part of table entries
pub const PG_OP_MMT:  u64 = 1<<4;  // update memory type part of table entries

#[derive(Debug, Copy, Clone)]
pub struct PagingConfig {
    pub modifier: u64,
    pub map_top:  u64,
    pub offset:   u64,
    pub pg_attr:  u64,
    pub tb_attr:  u64,
    pub pvl_msk:  u64,
    pub mmt_msk:  u64,
    pub large:    bool,
    pub pg_2m:    bool,
    pub pg_1g:    bool,
}

use info::InformationData;
use cpu::CPUSkillz;
use vmx::ept;

impl PagingConfig {
    // map virtual to system physical
    pub fn for_vmm(info: &InformationData) -> PagingConfig {
        PagingConfig {
            modifier: PG_OP_ADDR|PG_OP_PVL,
            map_top:  info.vmm.cpu.max_vaddr(),
            offset:   0,
            pg_attr:  PG_KRN|PG_RW|PG_P,
            tb_attr:  PG_KRN|PG_RW|PG_P,
            pvl_msk: (PG_NX|PG_USR|PG_RW),
            mmt_msk:  0,
            large:    true,
            pg_2m:    info.vmm.cpu.has_pg_2M(),
            pg_1g:    info.vmm.cpu.has_pg_1G(),
        }
    }

    // map guest physical to system physical
    pub fn for_vm(info: &InformationData) -> PagingConfig {
        PagingConfig {
            modifier: PG_OP_ADDR|PG_OP_PVL|PG_OP_MMT,
            map_top:  info.vm.cpu.max_paddr(),
            offset:   0,
            pg_attr:  ept::attr_dft(info),
            tb_attr:  ept::attr_dft(info),
            pvl_msk:  ept::attr_pvl_msk(),
            mmt_msk:  ept::attr_mmt_msk(),
            large:    true,
            pg_2m:    info.vm.cpu.has_pg_2M(),
            pg_1g:    info.vm.cpu.has_pg_1G(),
        }
    }
}

use core::slice;
use pool::{PageAllocator};

// Page Table Entriy trait
pub trait PTBEntry {
    type Alloc: PageAllocator;
    type Next: PTBMap<Alloc=Self::Alloc>; // Prevent map() type mismatch

    // must be implemented
    fn raw(&self)   -> u64;
    fn shift(&self) -> usize;
    fn set(&mut self, entry: u64);

    // default to native entries: may be overloaded
    fn present(&self) -> bool  { (self.raw() & PG_P ) != 0 }
    fn write(&self)   -> bool  { (self.raw() & PG_RW) != 0 }
    fn read(&self)    -> bool  { ! self.write() }
    fn execute(&self) -> bool  { (self.raw() & PG_NX) == 0 }


    // default implementation: should not be overloaded
    fn size(&self)    -> usize { 1 << self.shift() }

    fn may_be_large(&self) -> bool {
        (self.shift() == PG_2M_SHIFT || self.shift() == PG_1G_SHIFT)
    }

    fn is_large(&self) -> bool {
        ((self.raw() & PG_PS) != 0 && self.may_be_large())
    }

    fn is_page(&self) -> bool {
        (self.is_large() || self.shift() == PG_4K_SHIFT)
    }

    fn can_be_large(&self, conf: &PagingConfig) -> bool {
        (conf.large &&
         ((conf.pg_2m && self.shift() == PG_2M_SHIFT) ||
          (conf.pg_1g && self.shift() == PG_1G_SHIFT)))
    }

    fn can_be_a_page(&self, conf: &PagingConfig) -> bool {
        (self.can_be_large(conf) || self.shift() == PG_4K_SHIFT)
    }

    fn basic(&self, addr: u64, msk: u64, conf: &PagingConfig) -> u64 {
        let mut entry = 0;

        if (conf.modifier & PG_OP_ADDR) != 0 {
            entry |= addr & msk;
        }

        if (conf.modifier & PG_OP_OFF) != 0 {
            entry += conf.offset & msk;
        }

        // XXX: cache attributes (PAT, PCD, PWT, LPAT)
        // if (conf.modifier & PG_OP_CCH) != 0 {
        // }

        entry
    }

    fn erase(&mut self, msk: u64, conf: &PagingConfig) {
        let mut entry = self.raw();

        if (conf.modifier & PG_OP_ADDR) != 0 {
            entry &= !msk;
        }

        if (conf.modifier & PG_OP_PVL) != 0 {
            entry &= !conf.pvl_msk;
        }

        if (conf.modifier & PG_OP_MMT) != 0 {
            entry &= !conf.mmt_msk;
        }

        // XXX: cache attributes (PAT, PCD, PWT, LPAT)
        // if (conf.modifier & PG_OP_CCH) != 0 {
        // }

        if self.is_large() {
            entry &= !PG_PS;
        }

        self.set(entry);
    }


    // High level "full-range" operators
    // The range depends on entry level !
    //
    // ie. an L2 (PDE64) will map 2MB from addr, either using
    // a large page if supported, or mapping a whole L1 Table (PT64)
    fn map(&mut self, addr: u64, conf: &PagingConfig, alloc: &mut Self::Alloc) {
        if self.present() {
            self.unmap(addr, conf, alloc);
        }

        if self.can_be_a_page(conf) {
            self.set_page(addr, conf);
        } else {
            match alloc.get_page() {
                None => panic!("No memory for PTB (lv{}) !", self.shift()),
                Some(tbl_addr) => self.set_table(tbl_addr, conf),
            };

            self.as_table_mut().map(addr, conf, alloc);
        }
    }

    fn unmap(&mut self, addr: u64, conf: &PagingConfig, alloc: &mut Self::Alloc) {
        if self.present() {
            let mask = if !self.is_page() {
                self.as_table_mut().unmap(addr, conf, alloc);
                // XXX pool push page
                log!("--> pool.push_page() not implemented\n");
                addr_mask(PG_4K_SHIFT)
            } else {
                addr_mask(self.shift())
            };

            self.erase(mask, conf);
        }
    }

    fn remap(&mut self, addr: u64, conf: &PagingConfig, alloc: &mut Self::Alloc) {
        panic!("remap not implemented (lv{})\n", self.shift());
    }

    fn finest(&mut self, addr: u64, conf: &PagingConfig, alloc: &mut Self::Alloc) {
        let algn = if !pg_aligned(self.shift(), addr) {
            pg_align(self.shift(), addr)
        } else {
            addr
        };

        let mut nconf = *conf;
        nconf.large = false;
        nconf.modifier = PG_OP_ADDR|PG_OP_PVL|PG_OP_MMT;
        nconf.pg_attr = self.raw() & (conf.pvl_msk|conf.mmt_msk);

        /// XXX: self.erase() ? (always a large parge)
        self.unmap(algn, &nconf, alloc);
        self.map(algn, &nconf, alloc);
    }


    // Reach next entry, allocating table if needed
    fn next(&mut self, addr: u64, conf: &PagingConfig, alloc: &mut Self::Alloc)
            -> &mut <Self::Next as PTBMap>::Entry {

        if ! self.present() {
            let nt = match alloc.get_page() {
                None => panic!("No memory for next table (lv{}) !", self.shift()),
                Some(a) => a,
            };

            self.set_table(nt, conf);
        } else if self.is_large() {
            self.finest(addr, conf, alloc);
        }

        self.as_table_mut().at_mut(addr)
    }

    // Entry gives a Page Table

    fn table_addr(&self) -> u64 { self.raw() & addr_mask(PG_4K_SHIFT) }

    fn as_table(&self) -> &Self::Next {
        unsafe { & *(self.table_addr() as *const _) }
    }

    fn as_table_mut(&mut self) -> &mut Self::Next {
        unsafe { &mut *(self.table_addr() as *mut _) }
    }

    fn set_table(&mut self, addr: u64, conf: &PagingConfig) {
        let base = self.basic(addr, addr_mask(PG_4K_SHIFT), conf);
        self.set(base | conf.tb_attr);

        #[cfg(feature = "debug_paging")]
        log!("@{:#x} (lv{}) PTE.PTB 0x{:x} PTE = 0x{:08x}\n"
             ,{self as *const _ as *const u64 as u64}
             ,self.shift(), addr, self.raw());
    }



    // Entry gives a Page

    fn page_addr(&self) -> u64 { self.raw() & addr_mask(self.shift()) }

    fn as_page(&self) -> &[u8] {
        let ptr = self.page_addr() as *const u8;
        unsafe { slice::from_raw_parts(ptr, self.size()) }
    }

    fn as_page_mut(&mut self) -> &mut [u8] {
        let ptr = self.page_addr() as *mut u8;
        unsafe { slice::from_raw_parts_mut(ptr, self.size()) }
    }

    fn set_page(&mut self, addr: u64, conf: &PagingConfig) {
        let mut entry = self.basic(addr, addr_mask(self.shift()), conf);

        if (conf.modifier & PG_OP_PVL) != 0 {
            entry |= conf.pg_attr & conf.pvl_msk;
        } else {
            entry |= conf.pg_attr & !conf.pvl_msk;
        }

        if (conf.modifier & PG_OP_MMT) != 0 {
            entry |= conf.pg_attr & conf.mmt_msk;
        } else {
            entry |= conf.pg_attr & !conf.mmt_msk;
        }

        if self.may_be_large() {
            entry |= PG_PS;
        }

        self.set(entry);

        #[cfg(feature = "debug_paging")]
        log!("@{:#x} (lv{}) PTE.PG 0x{:x} - 0x{:x} PTE = 0x{:08x}\n"
             ,{self as *const _ as *const u64 as u64}
             ,self.shift(), addr, addr+(self.size() as u64), self.raw());
    }
}


/////////////////// Page Table definitions


// Generic MMU 64 bits table type
pub struct MMUTable64<T>([T;512]);

use core::ops::{Index, IndexMut};

impl<T> Index<usize> for MMUTable64<T> where T: PTBEntry {
    type Output = T;
    fn index(&self, idx: usize) -> &T { &self.0[idx] }
}

impl<T> IndexMut<usize> for MMUTable64<T> where T: PTBEntry {
    fn index_mut(&mut self, idx: usize) -> &mut T { &mut self.0[idx] }
}

pub trait PTBMap:
Index<usize,Output=<Self as PTBMap>::Entry> +
IndexMut<usize,Output=<Self as PTBMap>::Entry> {

    type Entry: PTBEntry<Alloc=Self::Alloc>; // prevent map() type mismatch
    type Alloc: PageAllocator;

    // XXX: crappy consider redesign !
    fn map(&mut self, addr: u64, conf: &PagingConfig, alloc: &mut Self::Alloc) {
        let mut pa = addr;
        let psz = self[0].size() as u64;
        for i in 0..512 {
            self[i].map(pa, conf, alloc);
            pa += psz;
        }
    }

    fn unmap(&mut self, addr: u64, conf: &PagingConfig, alloc: &mut Self::Alloc) {
        let mut pa = addr;
        let psz = self[0].size() as u64;
        for i in 0..512 {
            self[i].unmap(pa, conf, alloc);
            pa += psz;
        }
    }

    fn remap(&mut self, addr: u64, conf: &PagingConfig, alloc: &mut Self::Alloc) {
        let mut pa = addr;
        let psz = self[0].size() as u64;
        for i in 0..512 {
            self[i].remap(pa, conf, alloc);
            pa += psz;
        }
    }

    fn index_for(&self, addr: u64) -> usize {
        ((addr >> self[0].shift()) & 0x1ff) as usize
    }

    fn at(&self, addr: u64) -> &Self::Entry {
        let index = self.index_for(addr);
        &self[index]
    }

    fn at_mut(&mut self, addr: u64) -> &mut Self::Entry {
        let index = self.index_for(addr);
        &mut self[index]
    }

}
