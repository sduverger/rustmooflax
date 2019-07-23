use smem;
use vmm;
use vm;

pub struct InformationData {
    pub hwmm: smem::HardwareMemory,
    pub vmm: vmm::VMM,
    pub vm: vm::VM,
}
