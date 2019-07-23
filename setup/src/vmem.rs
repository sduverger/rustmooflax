use x86_64::PhysicalAddress;
use x86_64::registers::control_regs::cr3_write;

use share::vmx::ept;
use share::paging::ptb;
use share::mmap::PageMapper;
use share::info::info_data;

fn init_vmm() {
    let info = info_data();
    let pgconf = ptb::PagingConfig::for_vmm(info);
    let pool = &mut info.vmm.pool;

    info.vmm.pg.map(0, info.hwmm.phys, &pgconf, pool);
    unsafe { cr3_write(PhysicalAddress(info.vmm.pg.get_addr())) };
}

pub fn init() {
    init_vmm();
    ept::map::init();
}
