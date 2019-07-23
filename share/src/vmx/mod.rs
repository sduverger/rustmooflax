// Intel VMX extensions
pub mod insn;
pub mod regs;
pub mod vmcs;
pub mod ept;

pub enum ACTIVITY_STATE {
    Active = 0,
    Halt = 1,
    Shutdown = 2,
    Sipi = 3,
}
