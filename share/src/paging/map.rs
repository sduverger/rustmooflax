use core;
use paging::utils::*;
use paging::ptb::*;
use mmap::*;
use pool::PagePool;

///////////////// Native x86-64 implementation of Page Table Traits

#[derive(Default, Debug)]//, Copy, Clone)]
pub struct PML4Entry(u64);

impl PTBEntry for PML4Entry {
    type Next  = PDP;
    type Alloc = PagePool;

    fn raw(&self) -> u64 { self.0 }
    fn shift(&self) -> usize { PG_512G_SHIFT }
    fn set(&mut self, entry: u64) { self.0 = entry; }
}

pub type PML4 = MMUTable64<PML4Entry>;

impl PTBMap for PML4 {
    type Entry = PML4Entry;
    type Alloc = PagePool;
}


#[derive(Default, Debug)]//, Copy, Clone)]
pub struct PDPEntry(u64);

impl PTBEntry for PDPEntry {
    type Next  = PD64;
    type Alloc = PagePool;

    fn set(&mut self, entry: u64) { self.0 = entry; }
    fn raw(&self) -> u64 { self.0 }
    fn shift(&self) -> usize { PG_1G_SHIFT }
}

pub type PDP = MMUTable64<PDPEntry>;

impl PTBMap for PDP {
    type Entry = PDPEntry;
    type Alloc = PagePool;
}



#[derive(Default, Debug)]//, Copy, Clone)]
pub struct PD64Entry(u64);

impl PTBEntry for PD64Entry {
    type Next  = PT64;
    type Alloc = PagePool;

    fn set(&mut self, entry: u64) { self.0 = entry; }
    fn raw(&self) -> u64 { self.0 }
    fn shift(&self) -> usize { PG_2M_SHIFT }
}

pub type PD64 = MMUTable64<PD64Entry>;

impl PTBMap for PD64 {
    type Entry = PD64Entry;
    type Alloc = PagePool;
}



#[derive(Default, Debug)]//, Copy, Clone)]
pub struct PT64Entry(u64);

impl PTBEntry for PT64Entry {
    type Next  = PT64; // XXX: provide "None" instead ?
    type Alloc = PagePool;

    fn set(&mut self, entry: u64) { self.0 = entry; }
    fn raw(&self) -> u64 { self.0 }
    fn shift(&self) -> usize { PG_4K_SHIFT }
}

pub type PT64 = MMUTable64<PT64Entry>;

impl PTBMap for PT64 {
    type Entry = PT64Entry;
    type Alloc = PagePool;
}


// Specific VMM pre-allocated page tables
// pub struct VmmPageTables {
//     pub pml4: &'static mut PML4,
//     pub pdp:  &'static mut [PDP],
//     pub pd:   &'static mut [PD64],
// }


impl<'a> PageMapper for PagingEnv<'a, PML4> {
    type Allocator = PagePool; // XXX: create tmp allocator that ref to vmm pg mem

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
