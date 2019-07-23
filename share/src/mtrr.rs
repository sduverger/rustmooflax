use msr::*;

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct IA32MTRRPhysBase(u64);

    impl Debug;

    pub kind,_:7,0;
    pub base,_:63,12;
}

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct IA32MTRRPhysMask(u64);

    impl Debug;

    pub v,_:11;
    pub mask,_:63,12;
}

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct IA32MTRRCap(u64);

    impl Debug;

    pub u8, cnt,_:7,0;
    pub     fix,_:8;
    pub     wc,_:10;
    pub     smrr,_:11;
}

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct IA32MTRRDef(u64);

    impl Debug;

    pub kind,_:7,0;
    pub   fe,_:10;
    pub    e,_:11;
}

pub struct MTRRInfo {
    pub cap: IA32MTRRCap,
    pub def: IA32MTRRDef,
}

impl MTRRInfo {
    pub fn init(&mut self) {
        self.cap.0 = rdmsr(IA32_MTRRCAP);
        self.def.0 = rdmsr(IA32_MTRR_DEF_TYPE);
    }
}
