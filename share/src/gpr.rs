// We don't save RSP/RIP as they are in the VMCS
use utils::Raw64;

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct GPR64Context {
    pub r15: Raw64,
    pub r14: Raw64,
    pub r13: Raw64,
    pub r12: Raw64,
    pub r11: Raw64,
    pub r10: Raw64,
    pub r9:  Raw64,
    pub r8:  Raw64,
    pub rdi: Raw64,
    pub rsi: Raw64,
    pub rbp: Raw64,
    pub rbx: Raw64,
    pub rdx: Raw64,
    pub rcx: Raw64,
    pub rax: Raw64,
}
