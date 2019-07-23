use core;
use paging::utils::*;
use paging::ptb::*;
use vmx::ept::*;
use msr::*;
use mtrr::*;
use mmap::*;
use cpu::CPUSkillz;
use pool::PagePool;
use info::info_data;

///////////////// VMX EPT implementation of Page Table Traits

#[derive(Default, Debug)]//, Copy, Clone)]
pub struct PML4Entry(u64);
#[derive(Default, Debug)]//, Copy, Clone)]
pub struct PDPEntry(u64);
#[derive(Default, Debug)]//, Copy, Clone)]
pub struct PD64Entry(u64);
#[derive(Default, Debug)]//, Copy, Clone)]
pub struct PT64Entry(u64);

pub type PML4 = MMUTable64<PML4Entry>;
pub type PDP  = MMUTable64<PDPEntry>;
pub type PD64 = MMUTable64<PD64Entry>;
pub type PT64 = MMUTable64<PT64Entry>;


pub trait EPTEntry where Self: PTBEntry {
    fn present(&self) -> bool  { (self.raw() & PVL_RWX) != 0 }
    fn write(&self)   -> bool  { (self.raw() & PVL_W)   != 0 }
    fn read(&self)    -> bool  { (self.raw() & PVL_R)   != 0 }
    fn execute(&self) -> bool  { (self.raw() & PVL_X)   != 0 }
}

impl EPTEntry for PML4Entry {}
impl EPTEntry for PDPEntry  {}
impl EPTEntry for PD64Entry {}
impl EPTEntry for PT64Entry {}


impl PTBEntry for PML4Entry {
    type Next  = PDP;
    type Alloc = PagePool;

    fn raw(&self) -> u64 { self.0 }
    fn shift(&self) -> usize { PG_512G_SHIFT }
    fn set(&mut self, entry: u64) { self.0 = entry; }

    fn present(&self) -> bool  { <Self as EPTEntry>::present(self) }
    fn write(&self)   -> bool  { <Self as EPTEntry>::write(self)   }
    fn read(&self)    -> bool  { <Self as EPTEntry>::read(self)    }
    fn execute(&self) -> bool  { <Self as EPTEntry>::execute(self) }
}

impl PTBMap for PML4 {
    type Entry = PML4Entry;
    type Alloc = PagePool;
}

impl PTBEntry for PDPEntry {
    type Next  = PD64;
    type Alloc = PagePool;

    fn set(&mut self, entry: u64) { self.0 = entry; }
    fn raw(&self) -> u64 { self.0 }
    fn shift(&self) -> usize { PG_1G_SHIFT }

    fn present(&self) -> bool  { <Self as EPTEntry>::present(self) }
    fn write(&self)   -> bool  { <Self as EPTEntry>::write(self)   }
    fn read(&self)    -> bool  { <Self as EPTEntry>::read(self)    }
    fn execute(&self) -> bool  { <Self as EPTEntry>::execute(self) }
}

impl PTBMap for PDP {
    type Entry = PDPEntry;
    type Alloc = PagePool;
}

impl PTBEntry for PD64Entry {
    type Next  = PT64;
    type Alloc = PagePool;

    fn set(&mut self, entry: u64) { self.0 = entry; }
    fn raw(&self) -> u64 { self.0 }
    fn shift(&self) -> usize { PG_2M_SHIFT }

    fn present(&self) -> bool  { <Self as EPTEntry>::present(self) }
    fn write(&self)   -> bool  { <Self as EPTEntry>::write(self)   }
    fn read(&self)    -> bool  { <Self as EPTEntry>::read(self)    }
    fn execute(&self) -> bool  { <Self as EPTEntry>::execute(self) }
}

impl PTBMap for PD64 {
    type Entry = PD64Entry;
    type Alloc = PagePool;
}

impl PTBEntry for PT64Entry {
    type Next  = PT64; // XXX: provide "None" instead ?
    type Alloc = PagePool;

    fn set(&mut self, entry: u64) { self.0 = entry; }
    fn raw(&self) -> u64 { self.0 }
    fn shift(&self) -> usize { PG_4K_SHIFT }

    fn present(&self) -> bool  { <Self as EPTEntry>::present(self) }
    fn write(&self)   -> bool  { <Self as EPTEntry>::write(self)   }
    fn read(&self)    -> bool  { <Self as EPTEntry>::read(self)    }
    fn execute(&self) -> bool  { <Self as EPTEntry>::execute(self) }
}

impl PTBMap for PT64 {
    type Entry = PT64Entry;
    type Alloc = PagePool;
}



// High level page mapper
impl<'a> PageMapper for PagingEnv<'a, PML4> {
    type Allocator = PagePool;

    type L4T = PML4;
    type L3T = PDP;
    type L2T = PD64;
    type L1T = PT64;

    type L4E = PML4Entry;
    type L3E = PDPEntry;
    type L2E = PD64Entry;
    type L1E = PT64Entry;

    fn root(&self) -> &Self::L4T { self.root }
    fn root_mut(&mut self) -> &mut Self::L4T { self.root }
}

fn map_mtrr(base: u64, len: u64, attr: u64) {
    let info = info_data();
    let mut pgconf = PagingConfig::for_vm(info);
    let pool = &mut info.vmm.pool;

    info.vm.pg.unmap(base, base+len, &pgconf, pool);

    pgconf.pg_attr = attr | attr_pvl_dft();
    pgconf.tb_attr = attr | attr_pvl_dft();

    info.vm.pg.map(base, base+len, &pgconf, pool);
}


fn map_fixed_len_mtrr(mut base: u64, len: u64, msr: u64) -> u64 {
    for i in 0..8 {
        let kind = (msr >> (i*8)) as u8;
        let attr = (kind as u64) << 3;

        log!("MTRR fixed {:#x} - {:#x} type {}\n"
             ,base, base+len, kind);

        map_mtrr(base, len, attr);
        base += len;
    }

    base
}

fn map_fixed_mtrr() {
    let mut base = map_fixed_len_mtrr(0, 64*1024, rdmsr(IA32_MTRR_FIX64K_00000));

    base = map_fixed_len_mtrr(base, 16*1024, rdmsr(IA32_MTRR_FIX16K_80000));
    base = map_fixed_len_mtrr(base, 16*1024, rdmsr(IA32_MTRR_FIX16K_A0000));

    for i in IA32_MTRR_FIX4K_C0000..IA32_MTRR_FIX4K_F8000+1 {
        base = map_fixed_len_mtrr(base, 4*1024, rdmsr(i));
    }
}

fn map_variable_mtrr() {
    let info = info_data();

    for i in 0..info.vmm.cpu.mtrr.cap.cnt() {
        let m_idx  = (i*2) as u32;
        let m_base = IA32MTRRPhysBase( rdmsr(IA32_MTRR_PHYSBASE0 + m_idx) );
        let m_mask = IA32MTRRPhysMask( rdmsr(IA32_MTRR_PHYSBASE0 + m_idx + 1) );

        if m_mask.v() {
            let base = m_base.base()<<12;
            let mask = m_mask.mask()<<12;
            let size = info.vm.cpu.max_paddr() - mask + 1;
            let attr = m_base.kind() << 3;

            log!("MTRR variable {:#x} - {:#x} type {}\n"
                 ,base, base+size, m_base.kind());

            map_mtrr(base, size, attr);
        }
    }
}

pub fn init() {
    let info = info_data();
    let pgconf = PagingConfig::for_vm(info);
    let pool = &mut info.vmm.pool;

    info.vm.pg.asid = 1;
    info.vm.pg.map(0, info.hwmm.phys, &pgconf, pool);

    if info.vmm.cpu.mtrr.def.e() {
        map_variable_mtrr();

        if info.vmm.cpu.mtrr.def.fe() && info.vmm.cpu.mtrr.cap.fix() {
            map_fixed_mtrr();
        }
    }

    info.vm.pg.unmap(info.hwmm.area.start, info.hwmm.area.end, &pgconf, pool);
}
