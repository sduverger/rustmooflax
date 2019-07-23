use share::vmx::vmcs::access::Access;
use vmx::vmcs::setup::Setup;
use share::vmx::insn as vmx;

use share::rmode;
use share::mmap::PageMapper;
use share::utils::RawValue;
use share::info::info_data;

pub fn init() {
    let mut info = info_data();

    let revision = info.vmm.cpu.vmx.basic.vmcs_rev_id();
    let hw_vmcs = info.vm.vmc.region.get_addr();

    info.vm.vmc.region.set_revision_id(revision);

    vmx::vmclear(hw_vmcs);
    vmx::vmload(hw_vmcs);

    info.vm.vmcs.init();
    info.vm.vmcs.encode();
    info.vm.vmcs.commit();

    rmode::vm_set_entry(info.vm.vmcs.guest.rip.field().as_u64());
}
