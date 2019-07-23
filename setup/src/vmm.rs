use x86_64::registers::control_regs::Cr0 as CR0;
use x86_64::registers::control_regs::cr0 as cr0_read;
use x86_64::registers::control_regs::Cr4 as CR4;
use x86_64::registers::control_regs::cr4 as cr4_read;
use x86_64::registers::control_regs::{cr0_write, cr4_write};

use share::vmx::insn::vmxon;
use share::info::info_data;

pub fn init() {
    let info = info_data();

    let revision = info.vmm.cpu.vmx.basic.vmcs_rev_id();
    info.vmm.vmc.region.set_revision_id(revision);

    // XXX: don't use x86_64, rewrite set/get crX as u64
    let cr0 = cr0_read().bits() as u64;
    let cr0_fixed = info.vmm.cpu.vmx.fixed.cr0.mask_u64(cr0);

    let cr4 = cr4_read().bits() as u64;
    let cr4_fixed = info.vmm.cpu.vmx.fixed.cr4.mask_u64(cr4);
    let hw_vmcs = info.vmm.vmc.region.get_addr();

    unsafe {
        cr0_write(CR0::from_bits_truncate(cr0_fixed as usize));
        cr4_write(CR4::from_bits_truncate(cr4_fixed as usize));
        vmxon(hw_vmcs);
    }
}
