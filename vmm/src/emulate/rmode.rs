use vm;
use vmx::exit::VMMStatus;
use share::vmx::regs::EventType;
use share::vmx::vmcs::access::Access;
use share::info::InformationData;
use share::rmode;
use share::utils;
use share::utils::{RawValue, Raw64, ArithmeticMod16};
use core::mem;
use core::slice;

#[repr(C, packed)]
struct FarPointer {
    offset:   u32,
    selector: u16,
}

fn push(info: &mut InformationData, value: u16) -> VMMStatus {

    info.vm.vmcs.guest.rsp.as_mut().sub_mod16(2);

    let ss   = info.vm.vmcs.guest.ss.base.as_ref().as_u64();
    let addr = info.vm.vmcs.guest.rsp.as_ref().as_u64() + ss;

    let src = unsafe {
        let ptr = &value as *const _ as *const u8;
        slice::from_raw_parts(ptr, 2)
    };

    vm::mem::write(info, addr, src)
}

fn far_jump(info: &mut InformationData, target: &FarPointer) -> VMMStatus {
    info.vm.vmcs.guest.cs.sel.as_mut().0  = target.selector;
    info.vm.vmcs.guest.cs.base.as_mut().0 = target.selector as u64 * 16;
    info.vm.vmcs.guest.rip.as_mut().update_u32(target.offset);

    #[cfg(feature = "debug_rmode")]
    log!("far jump to {:#x}:{:#x}\n", target.selector, target.offset);

    VMMStatus::DoneLetRip
}

fn far_call(info: &mut InformationData,
            target: &FarPointer, isz: u16) -> VMMStatus {

    let cs = info.vm.vmcs.guest.cs.sel.as_ref().as_u16();
    match push(info, cs) {
        VMMStatus::Done => (),
        rc @ _ => return rc,
    }

    let mut rip = Raw64(info.vm.vmcs.guest.rip.as_ref().as_u64());
    rip.add_mod16(isz);

    match push(info, rip.as_u16()) {
        VMMStatus::Done => (),
        rc @ _ => return rc,
    }

    #[cfg(feature = "debug_rmode")]
    log!("far call saved {:#x}:{:#x}\n", cs, rip.as_u16());

    far_jump(info, target)
}

fn int_clear_flags(info: &mut InformationData) {
    let flags = info.vm.vmcs.guest.rflags.as_mut();

    flags.set_it(false);
    flags.set_ac(false);
    flags.set_tf(false);
    flags.set_rf(false);
}

pub fn interrupt(info: &mut InformationData, vector: u8, isz: u16) -> VMMStatus {
    #[cfg(feature = "debug_rmode")]
    log!("rmode int {:#x}\n", vector);

    if vector == rmode::BIOS_MISC_INTERRUPT {
        log!("rmode int 0x15 not implemented !\n");
        return VMMStatus::Fail
    }

    let mut entry = rmode::IVTEntry {..Default::default()};
    let addr = (vector as usize * mem::size_of::<rmode::IVTEntry>()) as u64;

    match vm::mem::read(info, addr, entry.as_mut_u8()) {
        VMMStatus::Done => (),
        rc @ _ => return rc,
    }

    let flags = info.vm.vmcs.guest.rflags.as_ref().as_u16();

    match push(info, flags) {
        VMMStatus::Done => (),
        rc @ _ => return rc,
    }

    int_clear_flags(info);

    let fptr  = FarPointer { offset: entry.ip as u32, selector: entry.cs };
    far_call(info, &fptr, isz)
}
