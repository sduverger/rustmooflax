// x86 Segmentation

use core::mem;
use x86_64::structures::tss::TaskStateSegment;

use interrupts::VmmIDT;
use utils;

// Segment Selector
pub const SELECTOR_TYPE_GDT: u16 = 0;
pub const SELECTOR_TYPE_LDT: u16 = 1;

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct SegmentSelector(u16);

    impl Debug;

    pub rpl,_:1,0;
    pub ti,_:2;
    pub index,_:15,3;
}

impl SegmentSelector {
    pub fn init(index: u16, ti: u16, rpl: u16) -> SegmentSelector {
        let raw: u16 = index<<3 | ti<<2 | rpl;
        SegmentSelector { 0: raw }
    }

    pub fn new_krn(index: u16) -> SegmentSelector {
        SegmentSelector::init(index, SELECTOR_TYPE_GDT, 0)
    }

    pub fn new_usr(index: u16) -> SegmentSelector {
        SegmentSelector::init(index, SELECTOR_TYPE_GDT, 3)
    }

    pub fn as_u16(&self) -> u16 { self.0 }
}

impl utils::RawValue for SegmentSelector {
    fn from_u32(x: u32) -> SegmentSelector { SegmentSelector(x as u16) }
    fn as_u64(&self) -> u64 { self.0 as u64 }
    fn update_u64(&mut self, v: u64) { self.0 = v as u16; }
}

// Segment Descriptor
pub const SEG_DESC_DATA_R:             u8 = 0x0;
pub const SEG_DESC_DATA_RA:            u8 = 0x1;
pub const SEG_DESC_DATA_RW:            u8 = 0x2;
pub const SEG_DESC_DATA_RWA:           u8 = 0x3;
pub const SEG_DESC_DATA_ER:            u8 = 0x4;
pub const SEG_DESC_DATA_ERA:           u8 = 0x5;
pub const SEG_DESC_DATA_ERW:           u8 = 0x6;
pub const SEG_DESC_DATA_ERWA:          u8 = 0x7;

pub const SEG_DESC_CODE_X:             u8 = 0x8;
pub const SEG_DESC_CODE_XA:            u8 = 0x9;
pub const SEG_DESC_CODE_XR:            u8 = 0xa;
pub const SEG_DESC_CODE_XRA:           u8 = 0xb;
pub const SEG_DESC_CODE_CX:            u8 = 0xc;
pub const SEG_DESC_CODE_CXA:           u8 = 0xd;
pub const SEG_DESC_CODE_CXR:           u8 = 0xe;
pub const SEG_DESC_CODE_CXRA:          u8 = 0xf;

pub const SEG_DESC_SYS_TSS_AVL_16:     u8 = 0x1;
pub const SEG_DESC_SYS_LDT:            u8 = 0x2;
pub const SEG_DESC_SYS_TSS_BUSY_16:    u8 = 0x3;
pub const SEG_DESC_SYS_CALL_GATE_16:   u8 = 0x4;
pub const SEG_DESC_SYS_TASK_GATE:      u8 = 0x5;
pub const SEG_DESC_SYS_INTR_GATE_16:   u8 = 0x6;
pub const SEG_DESC_SYS_TRAP_GATE_16:   u8 = 0x7;
pub const SEG_DESC_SYS_TSS_AVL_32:     u8 = 0x9;
pub const SEG_DESC_SYS_TSS_BUSY_32:    u8 = 0xb;
pub const SEG_DESC_SYS_CALL_GATE_32:   u8 = 0xc;
pub const SEG_DESC_SYS_INTR_GATE_32:   u8 = 0xe;
pub const SEG_DESC_SYS_TRAP_GATE_32:   u8 = 0xf;

pub const SEG_DESC_SYS_LDT_64:         u8 = 0x2;
pub const SEG_DESC_SYS_TSS_AVL_64:     u8 = 0x9;
pub const SEG_DESC_SYS_TSS_BUSY_64:    u8 = 0xb;
pub const SEG_DESC_SYS_CALL_GATE_64:   u8 = 0xc;
pub const SEG_DESC_SYS_INTR_GATE_64:   u8 = 0xe;
pub const SEG_DESC_SYS_TRAP_GATE_64:   u8 = 0xf;

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct SegmentDescriptor(u64);

    impl Debug;

    pub limit1,_:15,0;
    pub u16, base1,set_base1:31,16;
    pub u8, base2,set_base2:39,32;
    pub u8, kind,set_kind:43,40;
    pub s,_:44;
    pub dpl,_:46,45;
    pub p,set_p:47;
    pub limit2,_:51,48;
    pub l,_:53;
    pub d,_:54;
    pub g,_:55;
    pub u8, base3,set_base3:63,56;
}

type SegmentDescriptorLow = SegmentDescriptor;

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct SegmentDescriptorHigh(u64);

    impl Debug;

    pub u32, base4,set_base4:31,0;
}

#[repr(C, packed)]
pub struct SegmentDescriptor64 {
    low: SegmentDescriptorLow,
    high: SegmentDescriptorHigh,
}

impl SegmentDescriptor64 {
    pub fn setup_tss(&mut self, addr: u64) {
        self.low.0 = mem::size_of::<TaskStateSegment>() as u64;
        self.low.set_base1(addr as u16);
        self.low.set_base2((addr >> 16) as u8);
        self.low.set_base3((addr >> 24) as u8);
        self.low.set_kind(SEG_DESC_SYS_TSS_AVL_64);
        self.low.set_p(true);

        self.high.0 = 0;
        self.high.set_base4((addr >> 32) as u32);
    }
}

pub const CODE64_DESC: u64 = 0xaf9b000000ffff;
pub const CODE32_DESC: u64 = 0xcf9b000000ffff;
pub const DATA32_DESC: u64 = 0xcf93000000ffff;

pub const VMM_GDT_DESC_NR:  usize = 1+1+1+2*1; // null:code:data:tss64
pub const VMM_GDT_CODE_IDX: u16   = 1;
pub const VMM_GDT_DATA_IDX: u16   = 2;
pub const VMM_GDT_TSS_IDX:  u16   = 3;


pub trait DescriptorTable {
    type DescriptorType;

    fn table(&self) -> &[Self::DescriptorType];
    fn len(&self) -> u16;

    fn base(&self)  -> u64 { &self.table()[0] as *const _ as u64 }

    fn limit(&self) -> u16 {
        (self.len() * mem::size_of::<Self::DescriptorType>() as u16) - 1
    }
}

#[repr(C, packed)]
pub struct VmmGDT([SegmentDescriptor;VMM_GDT_DESC_NR]);

impl DescriptorTable for VmmGDT {
    type DescriptorType = SegmentDescriptor;

    fn table(&self) -> &[SegmentDescriptor] { &self.0 }
    fn len(&self)   -> u16 { VMM_GDT_DESC_NR as u16 }
}

impl VmmGDT {
    pub fn setup_u64(&mut self, index: u16, desc: u64) {
        if index > self.len() as u16 {
            panic!("Invalid GDT index {}", index);
        }

        self.0[index as usize].0 = desc;
    }

    pub fn setup_tss64(&mut self, index: u16, tss: &TaskStateSegment) {
        if index > self.len() - 1 { // takes 2 slots
            panic!("Invalid GDT TSS index {}", index);
        }

        unsafe {
            let tss_addr = tss as *const _ as u64;
            let tss_desc_addr = &self.0[index as usize] as *const _ as u64;
            let tss_desc = &mut *(tss_desc_addr as *mut SegmentDescriptor64);
            tss_desc.setup_tss(tss_addr);
        }
    }

    // pub fn show(&self) {
    //     let mut addr = self.base();

    //     log!("GDT base 0x{:x} limit {}\n",addr, self.limit());

    //     for desc in self.0.iter() {
    //         log!("0x{:x} GDT desc 0x{:x}\n", addr, desc.0);
    //         addr += 8;
    //     }
    // }
}


#[repr(C, packed)]
pub struct VmmSegmentation {
    pub gdt: VmmGDT,
    pub idt: VmmIDT,
    pub tss: TaskStateSegment,
}
