use x86_64::instructions::tables::{lgdt, load_tss, DescriptorTablePointer};
use x86_64::instructions::segmentation::*;
use x86_64::structures::gdt::SegmentSelector as SegSel;
use x86_64::PrivilegeLevel;

use share::segmentation::{VMM_GDT_CODE_IDX, VMM_GDT_DATA_IDX, VMM_GDT_TSS_IDX};
use share::segmentation::{CODE64_DESC, DATA32_DESC};
use share::segmentation::DescriptorTable;
use share::info::info_data;

pub fn init() {
    let info = info_data();

    info.vmm.seg.gdt.setup_u64(0, 0);
    info.vmm.seg.gdt.setup_u64(VMM_GDT_CODE_IDX, CODE64_DESC);
    info.vmm.seg.gdt.setup_u64(VMM_GDT_DATA_IDX, DATA32_DESC);
    info.vmm.seg.gdt.setup_tss64(VMM_GDT_TSS_IDX, &info.vmm.seg.tss);

    log!("GDT @ 0x{:x}\n", info.vmm.seg.gdt.base());

    unsafe {
        lgdt(&DescriptorTablePointer {
            base:  info.vmm.seg.gdt.base(),
            limit: info.vmm.seg.gdt.limit()
        });

        load_tss(SegSel::new(VMM_GDT_TSS_IDX, PrivilegeLevel::Ring0));

        set_cs(SegSel::new(VMM_GDT_CODE_IDX, PrivilegeLevel::Ring0));

        load_ss(SegSel::new(VMM_GDT_DATA_IDX, PrivilegeLevel::Ring0));
        load_ds(SegSel::new(VMM_GDT_DATA_IDX, PrivilegeLevel::Ring0));
        load_es(SegSel::new(VMM_GDT_DATA_IDX, PrivilegeLevel::Ring0));
        load_fs(SegSel::new(VMM_GDT_DATA_IDX, PrivilegeLevel::Ring0));
        load_gs(SegSel::new(VMM_GDT_DATA_IDX, PrivilegeLevel::Ring0));
    }

    // info.vmm.seg.gdt.show();
}
