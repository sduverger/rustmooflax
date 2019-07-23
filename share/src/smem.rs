use core::cmp;
use paging::utils as pgutils;

#[derive(Default, Copy, Clone)]
pub struct SecretArea {
    pub start: u64,
    pub end: u64,
    pub size: usize,
}

impl SecretArea {
    pub fn has_pfn(&self, pfn: usize) -> bool {
        let ps = pgutils::pfn(self.start);
        let pe = pgutils::pfn(self.end);
        (pfn >= ps && pfn < pe)
    }

    pub fn touch(&self, part: &[u8]) -> bool {
        let first = part as *const _ as *const u64 as u64;
        let last  = first + part.len() as u64 - 1;

        // XXX: overflow checked by runtime
        if first < last {
            if first >= self.end {
                return false;
            } else if last < self.start {
                return false;
            }
        }

        true
    }
}

#[derive(Default, Copy, Clone)]
pub struct HardwareMemory {
    pub area: SecretArea,
    pub ram:  u64,  // end of RAM
    pub phys: u64,  // end of physical accessible memory (i/o, >4GB ...)
}

impl HardwareMemory {
    pub fn setup(&mut self, sec: &SecretArea, ram_end: u64) {
        self.area = *sec;
        self.ram = ram_end;
        self.phys = cmp::max(self.ram, 1<<32);
    }

    pub fn get_total_frames(&self) -> usize {
        pgutils::pfn(self.ram)
    }

    pub fn show(&self) {
        log!("VMM area 0x{:x} - 0x{:x} {}KB\n",
             self.area.start, self.area.end, self.area.size/1024);
    }
}
