// Interrupts handling

use share::gpr::GPR64Context;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct InterruptContext {
    gpr:    GPR64Context,
    nr:     u64,
    err:    u64,
    rip:    u64,
    cs:     u64,
    rflags: u64,
    rsp:    u64,
    ss:     u64,
}

use core::fmt;

impl fmt::Debug for InterruptContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}
InterruptContext {{
    nr    : {},
    err   : {:#x},
    rip   : {:#x},
    cs    : {:#x},
    rflags: {:#x},
    rsp   : {:#x},
    ss    : {:#x}
}}",self.gpr, self.nr, self.err, self.rip, self.cs, self.rflags, self.rsp, self.ss)
    }
}

bitfield!{
    #[repr(C, packed)]
    pub struct IrqMsg(u16);

    impl Debug;

    pub u8, vector,set_vector:7,0;
    pub u8, pending,set_pending:15,8;

    pub preempt,set_preempt:16;
    pub rmode,set_rmode:17;
}

#[no_mangle]
pub extern "C" fn intr_hdlr(ctx: &InterruptContext) {
    panic!("VMM exception !\n{:#?}", ctx);
}
