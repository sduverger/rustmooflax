pub use self::api::*;
pub use self::itab::*;
pub use self::types::*;

mod api;
mod itab;
mod types;

use vmx::exit::VMMStatus;



// use disasm::{disassemble, ud};
// let mut dis: ud = unsafe { core::mem::zeroed() };
// unsafe { disassemble(&mut dis); }


pub unsafe fn disassemble(dis: &mut ud) -> VMMStatus {

    //get_insn(&vaddr, &mode)

    let data = [ 0x8B, 0x04, 0xB2 ];
    let mode = 16;

    ud_init(dis);
    ud_set_input_buffer(dis,
                        data.as_ptr(), // info->vm.cpu.insn_cache
                        data.len());    // sizeof(info->vm.cpu.insn_cache)

    ud_set_mode(dis, mode);
    ud_set_vendor(dis, ud_vendor::AMD);

    if ud_disassemble(dis) == 0 {
        return VMMStatus::Fail;
    }

    VMMStatus::Done
}
