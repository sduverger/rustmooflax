// Hardware/Virtual CPU definitions

use x86::current::cpuid;
use x86_64::registers::control_regs::Cr4 as CR4;
use x86_64::registers::control_regs::cr4 as cr4_read;
use x86_64::registers::control_regs::cr4_write;

use vmx::ept;
use vmx::regs::VMXInfo;
use mtrr::MTRRInfo;
use gpr::GPR64Context;
use msr;
use cr;

// This struct uses external types from extern crate we can't derive
// traits for them. We can use newtype pattern: see documentation "The
// Pattern to Implement External Traits on External Types"
pub struct CpuidCache {
    request: cpuid::CpuId,
    efi: cpuid::ExtendedFunctionInfo,
    feat: cpuid::FeatureInfo,
}

//#[derive(Default)]
pub struct HardwareCPU {
    cpuid: CpuidCache,
    pub vmx: VMXInfo,
    pub mtrr: MTRRInfo,
    paddr_sz: u8,
    vaddr_sz: u8,
    max_paddr: u64,
    max_vaddr: u64,
}

pub struct VirtualCPU {
    pub gpr: &'static mut GPR64Context,
    paddr_sz: u8,
    vaddr_sz: u8,
    max_paddr: u64,
    max_vaddr: u64,
    pg_2m: bool,
    pg_1g: bool,
    tlb: ept::VPID_INV_TYPE,
    tlb_g: ept::VPID_INV_TYPE,
}

pub trait CPUSkillz {
    fn max_paddr(&self) -> u64;
    fn max_vaddr(&self) -> u64;
    fn has_pg_2M(&self) -> bool { true }
    fn has_pg_1G(&self) -> bool;
}

impl CPUSkillz for HardwareCPU {
    fn max_paddr(&self) -> u64  { self.max_paddr }
    fn max_vaddr(&self) -> u64  { self.max_vaddr }
    fn has_pg_1G(&self) -> bool { self.cpuid.efi.has_1gib_pages() }
}

impl CPUSkillz for VirtualCPU {
    fn max_paddr(&self) -> u64  { self.max_paddr }
    fn max_vaddr(&self) -> u64  { self.max_vaddr }
    fn has_pg_2M(&self) -> bool { self.pg_2m }
    fn has_pg_1G(&self) -> bool { self.pg_1g }
}

impl VirtualCPU {
    pub fn setup(&mut self, hcpu: &HardwareCPU, gpr: u64) {
        self.gpr = unsafe { &mut *(gpr as *mut GPR64Context) };

        self.paddr_sz = hcpu.paddr_sz;
        self.vaddr_sz = hcpu.vaddr_sz;

        self.max_paddr = hcpu.max_paddr;
        self.max_vaddr = hcpu.max_vaddr;

        self.pg_2m = hcpu.vmx.ept.pg_2m();
        self.pg_1g = hcpu.vmx.ept.pg_1g();

        if hcpu.vmx.ept.invvpid_s() && hcpu.vmx.ept.invvpid_r() {
            self.tlb   = ept::VPID_INV_TYPE::Single;
            self.tlb_g = ept::VPID_INV_TYPE::SingleAll;
        } else if hcpu.vmx.ept.invvpid_a() {
            self.tlb   = ept::VPID_INV_TYPE::All;
            self.tlb_g = ept::VPID_INV_TYPE::All;
        } else {
            panic!("vmx ept has no valid invvpid type found");
        }
    }
}

impl HardwareCPU {
    pub fn has_osxsave(&self) -> bool {
        self.cpuid.feat.has_oxsave()
    }

    pub fn has_vmx(&self) -> bool {
        self.cpuid.feat.has_vmx()
    }

    fn set_cpuid(&mut self) {
        self.cpuid.request = cpuid::CpuId::new();

        self.cpuid.efi = match self.cpuid.request.get_extended_function_info() {
            Some(efi) => efi,
            None => panic!("Can't get cpuid extended function info")
        };

        self.cpuid.feat = match self.cpuid.request.get_feature_info() {
            Some(feat) => feat,
            None => panic!("Can't get cpuid feature info")
        };
    }

    fn set_max_addrs(&mut self) {
        self.paddr_sz = {
            match self.cpuid.efi.physical_address_bits() {
                Some(sz) => sz,
                None => panic!("can't get max physical address")
            }
        };

        self.vaddr_sz = {
            match self.cpuid.efi.linear_address_bits() {
                Some(sz) => sz,
                None => panic!("can't get max linear address")
            }
        };

        self.max_paddr = ((1 as u64) << self.paddr_sz) - 1;
        self.max_vaddr = ((1 as u64) << self.vaddr_sz) - 1;
    }

    fn enable_osxsave(&self) {
        unsafe {cr4_write(cr4_read()|CR4::ENABLE_OS_XSAVE)};
    }

    fn lock_enable_vmx(&self) {
        let raw = msr::rdmsr(msr::IA32_FEATURE_CONTROL);

        let mut feat =
            msr::IA32_FEAT_CTL::Flags::from_bits_truncate(raw);

        if feat.contains(msr::IA32_FEAT_CTL::LOCK) {
            if ! feat.contains(msr::IA32_FEAT_CTL::VMX) {
                panic!("vmx feature BIOS-locked");
            }
        } else { // lock enable vmx
            if ! feat.contains(msr::IA32_FEAT_CTL::VMX) {
                feat.insert(msr::IA32_FEAT_CTL::VMX);
            }

            feat.insert(msr::IA32_FEAT_CTL::LOCK);
            unsafe { msr::wrmsr(msr::IA32_FEATURE_CONTROL, feat.bits()) };
        }
    }

    pub fn show(&self) {
        log!("
- vmm cpu features
1GB pages support   : {:?}
osxsave enabled     : {:?}
max physical addr   : {:x}
max linear addr     : {:x}
"
             ,self.has_pg_1G()
             ,self.has_osxsave()
             ,self.max_paddr
             ,self.max_vaddr);
        log!("
- vmx cpu features
{:#?}
{:#?}
{:#?}
"
             ,self.vmx.basic
             ,self.vmx.misc
             ,self.vmx.ept);
    }

    #[cfg(feature = "setup")]
    pub fn setup(&mut self) {
        self.set_cpuid();
        self.set_max_addrs();

        if self.has_osxsave() {
            self.enable_osxsave();
        }

        if ! self.has_vmx() {
            panic!("vmx not supported");
        }

        self.lock_enable_vmx();
        self.vmx.init();
        self.mtrr.init();
        self.show();
    }
}


// #[cfg(feature = "setup")]
// pub fn init() -> HardwareCPU {
//     let mut cpu = HardwareCPU {..Default::default()};
//     cpu.setup();
//     cpu
// }

// #[cfg(feature = "setup")]
// impl Default for CpuidCache {
//     fn default() -> CpuidCache {
//         let req = cpuid::CpuId::new();

//         let efi = {
//             match req.get_extended_function_info() {
//                 Some(efi) => efi,
//                 None => panic!("Can't get cpuid extended function info")
//             }
//         };

//         let feat = {
//             match req.get_feature_info() {
//                 Some(feat) => feat,
//                 None => panic!("Can't get cpuid feature info")
//             }
//         };

//         CpuidCache {
//             request: req,
//             efi: efi,
//             feat: feat,
//         }
//     }
// }
