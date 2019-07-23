// x86 Interrupts

use segmentation::{VMM_GDT_CODE_IDX,SEG_DESC_SYS_INTR_GATE_64};
use segmentation::DescriptorTable;


bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct InterruptDescriptor(u64);

    impl Debug;

    pub u16, offset1,set_offset1:15,0;
    pub u16, selector,set_selector:31,16;
    pub u8, ist,_:34,32;
    pub u8, kind,set_kind:43,40;
    pub u8, dpl,set_dpl:46,45;
    pub p,set_p:47;
    pub u16, offset2,set_offset2:63,48;
}

type InterruptDescriptorLow = InterruptDescriptor;

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct InterruptDescriptorHigh(u64);

    impl Debug;

    pub u32, offset3,set_offset3:31,0;
}

#[repr(C, packed)]
pub struct InterruptDescriptor64 {
    low: InterruptDescriptorLow,
    high: InterruptDescriptorHigh,
}

impl InterruptDescriptor64 {

    pub fn setup(&mut self, sel: u16, hdl: u64, kind: u8, dpl: u8) {
        self.low.0  = 0;
        self.high.0 = 0;

        self.low.set_offset1(hdl as u16);
        self.low.set_offset2((hdl >> 16) as u16);
        self.high.set_offset3((hdl >> 32) as u32);

        self.low.set_selector(sel);
        self.low.set_kind(kind);
        self.low.set_dpl(dpl);
        self.low.set_p(true);
    }

    pub fn setup_gate64(&mut self, sel: u16, hdl: u64) {
        self.setup(sel, hdl, SEG_DESC_SYS_INTR_GATE_64, 0);
    }
}

#[repr(C, packed)]
pub struct VmmIDT(pub [InterruptDescriptor64;256]);

impl DescriptorTable for VmmIDT {
    type DescriptorType = InterruptDescriptor64;

    fn table(&self) -> &[InterruptDescriptor64] { &self.0 }
    fn len(&self) -> u16 { 256 }
}
