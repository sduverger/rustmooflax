// VM memory access

use vmx::exit::VMMStatus;
use share::info::InformationData;
use share::vmx::vmcs::access::Access as VMCSAccess;
use share::utils::RawValue;
use cpumode::{CPUMode, CPUState};
use share::cr;
use core::slice;

struct Access<'a> {    
    cr3:   u64,
    src:   &'a[u8],
    dst:   &'a mut[u8],
    write: bool,
}

fn access_system(info: &mut InformationData, access: &mut Access) -> VMMStatus {
    if access.write {
        if info.hwmm.area.touch(access.dst) { return VMMStatus::Fail }
    } else {
        if info.hwmm.area.touch(access.src) { return VMMStatus::Fail }
    }

    access.dst.copy_from_slice(access.src);
    VMMStatus::Done
}

fn access_physical(info: &mut InformationData, access: &mut Access) -> VMMStatus {
    // XXX: nested translation, out of ram, mm i/o, ...
    access_system(info, access)
}

fn validate_virtual(info: &mut InformationData, access: &mut Access) -> VMMStatus {
    VMMStatus::Fail
}

fn access_virtual(info: &mut InformationData, access: &mut Access) -> VMMStatus {
    log!("VM virtual mem access not implemented\n");
    // rebuild slice after page walk !
    VMMStatus::Fail
}

fn access_linear(info: &mut InformationData, access: &mut Access) -> VMMStatus {
    let cpu = CPUState::init(info);

    if cpu.is_real() || cpu.is_v8086() {
        return access_physical(info, access);
    }

    // XXX: segmentation checks

    if cpu.is_paged() {
        access_virtual(info, access)
    } else {
        access_physical(info, access)
    }
}

pub fn read(info: &mut InformationData, addr: u64, dst: &mut[u8]) -> VMMStatus {
    #[cfg(feature = "debug_vm_access_read")]
    log!("read {} bytes from VM memory from {:#x} to {:#x}\n"
         ,dst.len(), addr, dst.as_ptr() as u64);

    let src = unsafe {
        slice::from_raw_parts(addr as *const u8, dst.len())
    };

    let mut access = Access {
        cr3:   info.vm.vmcs.guest.cr3.as_ref().as_u64(),
        src:   src,
        dst:   dst,
        write: false
    };

    access_linear(info, &mut access)
}

pub fn write(info: &mut InformationData, addr: u64, src: &[u8]) -> VMMStatus {
    #[cfg(feature = "debug_vm_access_write")]
    log!("write {} bytes to VM memory from {:#x} to {:#x}\n"
         ,src.len(), src.as_ptr() as u64, addr);

    let dst = unsafe {
        slice::from_raw_parts_mut(addr as *mut u8, src.len())
    };

    let mut access = Access {
        cr3:   info.vm.vmcs.guest.cr3.as_ref().as_u64(),
        src:   src,
        dst:   dst,
        write: true
    };

    access_linear(info, &mut access)
}
