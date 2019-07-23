use core::mem;
use core::slice;
use paging::utils as pgutils;
use smem::HardwareMemory;

#[derive(Debug)]
pub enum FrameType {
    VM,
    VMM,
}

pub struct FrameDescriptor {
    pub ref_count: usize,
    pub flags: FrameType,
}

pub struct FrameRegistry {
    count: usize,
    desc: &'static mut[FrameDescriptor],
}

impl FrameRegistry {
    pub fn init(addr: u64, hwmm: &HardwareMemory) -> FrameRegistry {
        let ptr = addr as *mut FrameDescriptor;
        let cnt = hwmm.get_total_frames();
        let pfd = unsafe {slice::from_raw_parts_mut(ptr, cnt)};

        let mut registry = FrameRegistry {
            count: cnt,
            desc: pfd,
        };

        let mut pfn = 0;
        for dsc in registry.desc.iter_mut() {
            if hwmm.area.has_pfn(pfn) {
                dsc.ref_count = 1;
                dsc.flags = FrameType::VMM;
            } else {
                dsc.ref_count = 0;
                dsc.flags = FrameType::VM;
            }

            pfn += 1;
        }

        registry
    }

    pub fn get_desc(&self, addr: u64) -> Option<&FrameDescriptor> {
        let n = pgutils::pfn(addr);

        if n < self.count {
            Some(&self.desc[n])
        } else { None }
    }

    // get memory size of our frame descriptor slice
    pub fn size(&self) -> usize {
        mem::size_of_val(self.desc)
    }
}
