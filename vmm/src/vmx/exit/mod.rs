// submodules implementing specific vmexit handlers
mod excp;
mod reason;

use vmx::exit::reason::BasicReason;
use vmx::vmcs::commit::Commit;
use share::vmx::vmcs::access::Access;
use share::utils::RawValue;
use share::info::InformationData;
use share::info::info_data;

#[derive(Debug, Copy, Clone)]
pub enum VMMStatus {
    Done,
    DoneLetRip,
    Fail,
    Fault,
    Native,
    Ignore,
    Internal,
    Partial,
}

#[no_mangle]
pub extern fn vmresume_failure(vmx_err: u32) -> ! {
    panic!("vmresume failed {}", vmx_err);
}

#[no_mangle]
pub extern fn vmexit_handler() {
    let info  = info_data();
    let basic = info.vm.vmcs.exit.reason.as_ref().basic();

    if let VMMStatus::Fail = BasicReason::resolve(info, basic) {
        panic!("vm-exit failure !\n{:#?}\n", info.vm.vmcs.exit.reason.as_ref());
    }

    info.vm.vmcs.commit();
}
