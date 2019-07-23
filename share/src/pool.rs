use rlibc::memset;
use paging::utils as pgutils;

pub trait PageAllocator {
    fn get_page(&mut self) -> Option<u64>;
    //pub release_page(&mut self);
}


pub struct PagePool {
    pub start: u64,
    pub used: usize,
    pub size: usize,
}

impl PageAllocator for PagePool {

    // XXX: should implement free/used/ linked lists
    fn get_page(&mut self) -> Option<u64> {
        if self.used < self.size - pgutils::PG_4KB {
            let addr = self.start + (self.used as u64);
            self.used += pgutils::PG_4KB;
            unsafe { memset(addr as *mut u8, 0, pgutils::PG_4KB); }

            log!("Pool: alloc {:#x} used {:#x} size {:#x}\n"
                 , addr, self.used, self.size);

            Some(addr)
        } else {
            None
        }
    }
}
