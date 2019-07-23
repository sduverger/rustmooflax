use cpu;
use smap;
use vmx::vmcs;
use vmx::ept::map as eptmap;
use paging::ptb as pgptb;

pub struct VM {
    pub cpu:  cpu::VirtualCPU,
    pub smap: smap::SystemMap,
    pub vmc:  &'static mut vmcs::VmHardwareVMCS,
    pub vmcs: vmcs::VMCS,
    pub pg:   pgptb::PagingEnv<'static, eptmap::PML4>,
}
