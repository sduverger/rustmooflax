use x86_64::instructions::tables::{lidt, DescriptorTablePointer};

use share::segmentation::SegmentSelector as SegSel;
use share::segmentation::VMM_GDT_CODE_IDX;
use share::segmentation::DescriptorTable;
use share::info::info_data;

const VMM_IDT_ISR_ALIGN: u64 = 16;

pub fn init() {
    let info = info_data();

    log!("IDT @ 0x{:x}\n", info.vmm.seg.idt.base());

    let cs = SegSel::new_krn(VMM_GDT_CODE_IDX).as_u16();
    let mut hdl = info.vmm.base + VMM_IDT_ISR_ALIGN;

    for desc in info.vmm.seg.idt.0.iter_mut() {
        desc.setup_gate64(cs, hdl);
        hdl += VMM_IDT_ISR_ALIGN;
    }

    unsafe {
        lidt(&DescriptorTablePointer {
            base:  info.vmm.seg.idt.base(),
            limit: info.vmm.seg.idt.limit()
        });
    }
}
