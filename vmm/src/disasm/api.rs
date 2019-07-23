use disasm::types::{ud, ud_operand, ud_vendor};
use disasm::itab::ud_mnemonic_code;

extern "C" {
    pub fn ud_init(ud: *mut ud);
    pub fn ud_set_mode(ud: *mut ud, mode: u8);
    pub fn ud_set_input_buffer(ud: *mut ud, data: *const u8, len: usize);
    pub fn ud_insn_len(ud: *const ud) -> u32;
    pub fn ud_disassemble(ud: *mut ud) -> u32;
    pub fn ud_set_vendor(ud: *mut ud, vendor: ud_vendor);
}
