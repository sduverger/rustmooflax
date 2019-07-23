use core::slice;
use multiboot::{Multiboot, MemoryType};

#[derive(PartialEq)]
pub enum SystemMapEntryType {
    Available = 1, // memory, available to OS
    Reserved  = 2, // reserved, not available (rom, mem map dev)
    ACPI      = 3, // ACPI Reclaim Memory
    NVS       = 4, // ACPI NVS Memory
}

impl SystemMapEntryType {
    pub fn from_mbi(mtype: MemoryType) -> SystemMapEntryType {
        match mtype {
            MemoryType::RAM => SystemMapEntryType::Available,
            _ => SystemMapEntryType::Reserved,
        }
    }
}

pub struct SystemMapEntry {
    pub base: u64,
    pub len: usize,
    pub typ: SystemMapEntryType,
}

pub struct SystemMap {
    count: usize,
    entries: &'static mut[SystemMapEntry],
}

impl SystemMap {
    #[cfg(feature = "setup")]
    pub fn init(addr: u64, cnt: usize, end: u64, mbi: &Multiboot) -> SystemMap {
        let ptr = addr as *mut SystemMapEntry;
        let sme = unsafe {slice::from_raw_parts_mut(ptr, cnt)};
        let mut smap = SystemMap {
            count: cnt,
            entries: sme,
        };

        let mmaps = match mbi.memory_regions() {
            None => panic!("No multiboot memory maps !"),
            Some(mm) => mm,
        };

        let mut idx = 0;
        for m in mmaps {
            let mut sme = &mut smap.entries[idx];
            sme.base = m.base_address();
            sme.typ = SystemMapEntryType::from_mbi(m.memory_type());

            if SystemMapEntryType::Available == sme.typ && sme.base == 1<<20 {
                sme.len = (end - sme.base) as usize;
            } else {
                sme.len = m.length() as usize;
            }
        }

        smap
    }
}
