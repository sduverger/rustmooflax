use x86_64::registers::control_regs::cr0 as cr0_read;
use x86_64::registers::control_regs::cr2 as cr2_read;
use x86_64::registers::control_regs::cr3 as cr3_read;
use x86_64::registers::control_regs::cr4 as cr4_read;

use share::cr;
use share::cr::cr2_write;
use share::msr;

use share::vmx::ACTIVITY_STATE;
use share::vmx::ept;
use share::vmx::regs::*;
use share::vmx::vmcs::*;
use share::vmx::vmcs::enc::*;
use share::vmx::vmcs::access::Access;

use share::rmode;
use share::exceptions as excp;
use share::segmentation::{VMM_GDT_CODE_IDX,VMM_GDT_DATA_IDX,VMM_GDT_TSS_IDX};
use share::segmentation::SegmentSelector as SegSel;
use share::segmentation::DescriptorTable;
use share::utils::RawValue;
use share::info::info_data;

pub trait Setup {
    // Every VMCS sub structures should implement this trait
    fn init(&mut self)   {}
    fn commit(&mut self) {}
    fn encode(&mut self);
}

impl Setup for VMCS {
    fn init(&mut self) {
        #[cfg(feature = "debug_vmcs_setup")]
        log!("vmcs ctrl init\n");
        self.ctrl.init();
        #[cfg(feature = "debug_vmcs_setup")]
        log!("vmcs host init\n");
        self.host.init();
        #[cfg(feature = "debug_vmcs_setup")]
        log!("vmcs guest init\n");
        self.guest.init();
    }

    fn encode(&mut self) {
        self.ctrl.encode();
        self.exit.encode();
        self.host.encode();
        self.guest.encode();
    }

    fn commit(&mut self) {
        self.ctrl.commit();
        self.host.commit();
        self.guest.commit();
    }
}

impl Setup for Exit {
    // 16, 64, 32, Natural fields
    fn encode(&mut self) {
        self.guest_physical.set_encoding(EXIT_INFO_GUEST_PHYSICAL_ADDR);

        self.vmx_insn_err.set_encoding(EXIT_INFO_VM_INSN_ERROR);
        self.reason.set_encoding(EXIT_INFO_REASON);
        self.int_info.set_encoding(EXIT_INFO_VMEXIT_INT_INFO);
        self.int_err_code.set_encoding(EXIT_INFO_VMEXIT_INT_ERR);
        self.idt_info.set_encoding(EXIT_INFO_IDT_VECT_INFO);
        self.idt_err_code.set_encoding(EXIT_INFO_IDT_VECT_ERR);
        self.insn_len.set_encoding(EXIT_INFO_VMEXIT_INSN_LEN);
        self.insn_info.set_encoding(EXIT_INFO_VMEXIT_INSN_INFO);

        self.qualification.set_encoding(EXIT_INFO_QUALIFICATION);
        self.io_rcx.set_encoding(EXIT_INFO_IO_RCX);
        self.io_rsi.set_encoding(EXIT_INFO_IO_RSI);
        self.io_rdi.set_encoding(EXIT_INFO_IO_RDI);
        self.io_rip.set_encoding(EXIT_INFO_IO_RIP);
        self.guest_linear.set_encoding(EXIT_INFO_GUEST_LINEAR_ADDR);
    }
}

impl Setup for Ctl {
    fn init(&mut self) {
        self.entry.init();
        self.exec.init();
        self.exit.init();
    }

    fn encode(&mut self) {
        self.entry.encode();
        self.exec.encode();
        self.exit.encode();
    }

    fn commit(&mut self) {
        self.entry.commit();
        self.exec.commit();
        self.exit.commit();
    }
}

impl Setup for EntryCtl {
    fn init(&mut self) {
        let info = info_data();

        let entry = self.entry.field_mut();
        entry.set_load_ia32_perf(true);
        entry.set_load_ia32_pat(true);
        entry.set_load_ia32_efer(true);

        // XXX: TODO
        //entry.set_load_dbgctl(true);
        //entry.set_load_ia32_bnd(true);

        self.msr_load_addr.set_field_value(info.vm.vmc.entry_load.get_addr());
    }

    // 16, 64, 32, Natural fields
    fn encode(&mut self) {
        self.msr_load_addr.set_encoding(ENTRY_CTRL_MSR_LOAD_ADDR);

        self.entry.set_encoding(ENTRY_CTRLS);
        self.msr_load_cnt.set_encoding(ENTRY_CTRL_MSR_LOAD_COUNT);
        self.int_info.set_encoding(ENTRY_CTRL_INT_INFO);
        self.int_err_code.set_encoding(ENTRY_CTRL_EXCP_ERR_CODE);
        self.insn_len.set_encoding(ENTRY_CTRL_INSN_LEN);
    }

    fn commit(&mut self) {
        self.msr_load_addr.force_flush();

        let info = info_data();
        self.entry.set_fixed(info.vmm.cpu.vmx.fixed.entry);
        self.entry.force_flush();

        #[cfg(feature = "debug_vmcs_setup")]
        log!("entry\n1fix1 0b{:032b}\n0fix0 0b{:032b}\n      0b{:032b}\n",
             self.entry.fixed().allow_0.0,
             self.entry.fixed().allow_1.0,
             self.entry.field().0
        );


        self.msr_load_cnt.force_flush();
        self.int_info.force_flush();
        self.int_err_code.force_flush();
        self.insn_len.force_flush();
    }
}


impl Setup for ExecCtl {
    fn init(&mut self) {
        let info = info_data();

        let proc1 = self.proc1.field_mut();
        proc1.set_tsc(true);
        //proc1.set_cr3l(true);
        proc1.set_usio(true);
        proc1.set_umsr(true);
        proc1.set_proc2(true);

        let proc2 = self.proc2.field_mut();
        proc2.set_uguest(true);
        // proc2.set_dt(true);
        proc2.set_rdtscp(true);
        proc2.set_ept(true);
        proc2.set_vpid(true);

        let eptp = self.eptp.field_mut();
        eptp.update_u64(info.vm.pg.get_addr());
        eptp.set_cache(ept::MMT_WB);
        eptp.set_pwl(3);

        self.vpid.set_field_value(info.vm.pg.asid as u64);

        self.excp_bitmap.set_field_value((1<<excp::GP|1<<excp::MC) as u64);

        self.pf_err_msk.set_field_value(0);
        self.pf_err_mch.set_field_value(0);

        let cr0_mask = self.cr0_mask.field_mut();
        // cr0_mask.set_pe(true);
        // cr0_mask.set_cd(true);
        // cr0_mask.set_pg(true);

        let cr4_mask = self.cr4_mask.field_mut();
        // cr4_mask.set_mce(true);
        // cr4_mask.set_pae(true);
        // cr4_mask.set_pse(true);
        // cr4_mask.set_pge(true);
        // cr4_mask.set_vmxe(true);

        self.ioA_bitmap.set_field_value(info.vm.vmc.ioA_map.get_addr());
        self.ioB_bitmap.set_field_value(info.vm.vmc.ioB_map.get_addr());

        info.vm.vmc.msr_map.deny(msr::IA32_MTRR_DEF_TYPE);
        info.vm.vmc.msr_map.deny(msr::IA32_EFER);

        self.msr_bitmap.set_field_value(info.vm.vmc.msr_map.get_addr());
    }

    // 16, 64, 32, Natural fields
    fn encode(&mut self) {
        self.vpid.set_encoding(EXEC_CTRL_VPID);
        self.eptp_idx.set_encoding(EXEC_CTRL_EPTP_IDX);

        self.ioA_bitmap.set_encoding(EXEC_CTRL_ADDR_IO_MAP_A);
        self.ioB_bitmap.set_encoding(EXEC_CTRL_ADDR_IO_MAP_B);
        self.msr_bitmap.set_encoding(EXEC_CTRL_ADDR_MSR_MAP);
        self.executive_vmcs_ptr.set_encoding(EXEC_CTRL_VMCS_PTR);
        self.tsc_offset.set_encoding(EXEC_CTRL_TSC_OFFSET);
        self.vapic_addr.set_encoding(EXEC_CTRL_VAPIC_PAGE_ADDR);
        self.apic_addr.set_encoding(EXEC_CTRL_APIC_PAGE_ADDR);
        self.eptp.set_encoding(EXEC_CTRL_EPTP);
        self.pml_addr.set_encoding(EXEC_CTRL_PML_ADDR);
        self.posted_int.set_encoding(EXEC_CTRL_POSTED_INT);
        self.vm_func.set_encoding(EXEC_CTRL_VM_FUNC);
        self.eptp_list.set_encoding(EXEC_CTRL_EPTP_LIST);
        self.vmread_bitmap.set_encoding(EXEC_CTRL_VMREAD_MAP);
        self.vmwrite_bitmap.set_encoding(EXEC_CTRL_VMWRITE_MAP);
        self.vmx_excp_addr.set_encoding(EXEC_CTRL_VMX_EXCP_ADDR);
        self.xss_bitmap.set_encoding(EXEC_CTRL_XSS_MAP);
        self.encls_bitmap.set_encoding(EXEC_CTRL_ENCLS_MAP);

        self.pin.set_encoding(EXEC_CTRL_PINBASED);
        self.proc1.set_encoding(EXEC_CTRL_PROCBASED_1);
        self.proc2.set_encoding(EXEC_CTRL_PROCBASED_2);
        self.excp_bitmap.set_encoding(EXEC_CTRL_EXCP_BITMAP);
        self.pf_err_msk.set_encoding(EXEC_CTRL_PF_ERR_MASK);
        self.pf_err_mch.set_encoding(EXEC_CTRL_PF_ERR_MATCH);
        self.cr3_target_cnt.set_encoding(EXEC_CTRL_CR3_TARGET_COUNT);
        self.tpr_threshold.set_encoding(EXEC_CTRL_TPR_THRESHOLD);
        self.ple_gap.set_encoding(EXEC_CTRL_PLE_GAP);
        self.ple_win.set_encoding(EXEC_CTRL_PLE_WIN);

        self.cr0_mask.set_encoding(EXEC_CTRL_CR0_GUEST_HOST_MASK);
        self.cr4_mask.set_encoding(EXEC_CTRL_CR4_GUEST_HOST_MASK);
        self.cr0_read_shadow.set_encoding(EXEC_CTRL_CR0_READ_SHADOW);
        self.cr4_read_shadow.set_encoding(EXEC_CTRL_CR4_READ_SHADOW);
        self.cr3_target_0.set_encoding(EXEC_CTRL_CR3_TARGET_0);
        self.cr3_target_1.set_encoding(EXEC_CTRL_CR3_TARGET_1);
        self.cr3_target_2.set_encoding(EXEC_CTRL_CR3_TARGET_2);
        self.cr3_target_3.set_encoding(EXEC_CTRL_CR3_TARGET_3);
    }

    fn commit(&mut self) {
        let info = info_data();

        self.pin.set_fixed(info.vmm.cpu.vmx.fixed.pin);
        self.pin.force_flush();
        #[cfg(feature = "debug_vmcs_setup")]
        log!("pin\n1fix1 0b{:032b}\n0fix0 0b{:032b}\n      0b{:032b}\n",
             self.pin.fixed().allow_0.0,
             self.pin.fixed().allow_1.0,
             self.pin.field().0
        );

        self.proc1.set_fixed(info.vmm.cpu.vmx.fixed.proc1);
        self.proc1.force_flush();
        #[cfg(feature = "debug_vmcs_setup")]
        log!("proc1\n1fix1 0b{:032b}\n0fix0 0b{:032b}\n      0b{:032b}\n",
             self.proc1.fixed().allow_0.0,
             self.proc1.fixed().allow_1.0,
             self.proc1.field().0
        );

        self.proc2.set_fixed(info.vmm.cpu.vmx.fixed.proc2);
        self.proc2.force_flush();
        #[cfg(feature = "debug_vmcs_setup")]
        log!("proc2\n1fix1 0b{:032b}\n0fix0 0b{:032b}\n      0b{:032b}\n",
             self.proc2.fixed().allow_0.0,
             self.proc2.fixed().allow_1.0,
             self.proc2.field().0
        );


        self.vpid.force_flush();
        // self.eptp_idx.force_flush();

        self.ioA_bitmap.force_flush();
        self.ioB_bitmap.force_flush();
        self.msr_bitmap.force_flush();
        self.executive_vmcs_ptr.force_flush();
        self.tsc_offset.force_flush();
        self.eptp.force_flush();

        if info.vmm.cpu.vmx.fixed.proc1.allow_1.tprs() {
            self.vapic_addr.force_flush();
        }

        if info.vmm.cpu.vmx.fixed.proc2.allow_1.vapic() {
            self.apic_addr.force_flush();
        }

        // self.pml_addr.force_flush();
        // self.posted_int.force_flush();
        // self.vm_func.force_flush();
        // self.eptp_list.force_flush();
        // self.vmread_bitmap.force_flush();
        // self.vmwrite_bitmap.force_flush();
        // self.vmx_excp_addr.force_flush();
        // self.xss_bitmap.force_flush();
        // self.encls_bitmap.force_flush();

        self.excp_bitmap.force_flush();
        self.pf_err_msk.force_flush();
        self.pf_err_mch.force_flush();
        self.cr3_target_cnt.force_flush();
        self.tpr_threshold.force_flush();
        // self.ple_gap.force_flush();
        // self.ple_win.force_flush();

        self.cr0_mask.force_flush();
        self.cr4_mask.force_flush();
        self.cr0_read_shadow.force_flush();
        self.cr4_read_shadow.force_flush();
        self.cr3_target_0.force_flush();
        self.cr3_target_1.force_flush();
        self.cr3_target_2.force_flush();
        self.cr3_target_3.force_flush();
    }
}

impl Setup for  ExitCtl {
    fn init(&mut self) {
        let info = info_data();

        let exit = self.exit.field_mut();
        exit.set_ack_int(true);
        exit.set_host_lmode(true);

        exit.set_load_ia32_perf(true);
        exit.set_load_ia32_pat(true);
        exit.set_load_ia32_efer(true);

        exit.set_save_ia32_pat(true);
        exit.set_save_ia32_efer(true);

        // XXX: TODO
        //exit.set_save_dbgctl(true);
        //exit.set_clear_bnd(true);

        self.msr_store_addr.set_field_value(info.vm.vmc.exit_store.get_addr());
        self.msr_load_addr.set_field_value(info.vm.vmc.exit_load.get_addr());

    }

    // 16, 64, 32, Natural fields
    fn encode(&mut self) {
        self.msr_store_addr.set_encoding(EXIT_CTRL_MSR_STORE_ADDR);
        self.msr_load_addr.set_encoding(EXIT_CTRL_MSR_LOAD_ADDR);

        self.exit.set_encoding(EXIT_CTRLS);
        self.msr_store_cnt.set_encoding(EXIT_CTRL_MSR_STORE_COUNT);
        self.msr_load_cnt.set_encoding(EXIT_CTRL_MSR_LOAD_COUNT);
    }

    fn commit(&mut self) {
        self.msr_store_addr.force_flush();
        self.msr_load_addr.force_flush();

        let info = info_data();
        self.exit.set_fixed(info.vmm.cpu.vmx.fixed.exit);
        self.exit.force_flush();
        #[cfg(feature = "debug_vmcs_setup")]
        log!("exit\n1fix1 0b{:032b}\n0fix0 0b{:032b}\n      0b{:032b}\n",
             self.exit.fixed().allow_0.0,
             self.exit.fixed().allow_1.0,
             self.exit.field().0
        );

        self.msr_store_cnt.force_flush();
        self.msr_load_cnt.force_flush();
    }
}

impl Setup for Host {
    fn init(&mut self) {
        let info = info_data();

        self.cr0.set_field_value(cr0_read().bits() as u64);
        self.cr3.set_field_value(cr3_read().0);
        self.cr4.set_field_value(cr4_read().bits() as u64);

        #[cfg(feature = "debug_vmcs_setup")]
        log!("rdmsr IA32 SYSENTER CS\n");
        self.ia32_sysenter_cs.set_field_value(msr::rdmsr(msr::IA32_SYSENTER_CS));
        #[cfg(feature = "debug_vmcs_setup")]
        log!("rdmsr IA32 SYSENTER EIP\n");
        self.ia32_sysenter_eip.set_field_value(msr::rdmsr(msr::IA32_SYSENTER_EIP));
        #[cfg(feature = "debug_vmcs_setup")]
        log!("rdmsr IA32 SYSENTER ESP\n");
        self.ia32_sysenter_esp.set_field_value(msr::rdmsr(msr::IA32_SYSENTER_ESP));

        #[cfg(feature = "debug_vmcs_setup")]
        log!("rdmsr IA32 PERF\n");
        self.ia32_perf.set_field_value(msr::rdmsr(msr::IA32_PERF_GLOBAL_CTRL));
        #[cfg(feature = "debug_vmcs_setup")]
        log!("rdmsr IA32 PAT\n");
        self.ia32_pat.set_field_value(msr::rdmsr(msr::IA32_PAT));
        #[cfg(feature = "debug_vmcs_setup")]
        log!("rdmsr IA32 EFER\n");
        self.ia32_efer.set_field_value(msr::rdmsr(msr::IA32_EFER));

        self.rsp.set_field_value(info.vmm.stack);
        self.rip.set_field_value(info.vmm.entry);

        self.cs.set_field_value(SegSel::new_krn(VMM_GDT_CODE_IDX).as_u64());

        self.ss.set_field_value(SegSel::new_krn(VMM_GDT_DATA_IDX).as_u64());
        self.ds.set_field_value(SegSel::new_krn(VMM_GDT_DATA_IDX).as_u64());
        self.es.set_field_value(SegSel::new_krn(VMM_GDT_DATA_IDX).as_u64());
        self.fs.set_field_value(SegSel::new_krn(VMM_GDT_DATA_IDX).as_u64());
        self.gs.set_field_value(SegSel::new_krn(VMM_GDT_DATA_IDX).as_u64());

        self.tr.set_field_value(SegSel::new_krn(VMM_GDT_TSS_IDX).as_u64());

        self.gdtr_base.set_field_value(info.vmm.seg.gdt.base());
        self.idtr_base.set_field_value(info.vmm.seg.idt.base());
    }

    // 16, 64, 32, Natural fields
    fn encode(&mut self) {
        self.es.set_encoding(HOST_STATE_ES_SEL);
        self.cs.set_encoding(HOST_STATE_CS_SEL);
        self.ss.set_encoding(HOST_STATE_SS_SEL);
        self.ds.set_encoding(HOST_STATE_DS_SEL);
        self.fs.set_encoding(HOST_STATE_FS_SEL);
        self.gs.set_encoding(HOST_STATE_GS_SEL);
        self.tr.set_encoding(HOST_STATE_TR_SEL);

        self.ia32_pat.set_encoding(HOST_STATE_IA32_PAT);
        self.ia32_efer.set_encoding(HOST_STATE_IA32_EFER);
        self.ia32_perf.set_encoding(HOST_STATE_IA32_PERF_GLOBAL);

        self.ia32_sysenter_cs.set_encoding(HOST_STATE_IA32_SYSENTER_CS);

        self.cr0.set_encoding(HOST_STATE_CR0);
        self.cr3.set_encoding(HOST_STATE_CR3);
        self.cr4.set_encoding(HOST_STATE_CR4);
        self.fs_base.set_encoding(HOST_STATE_FS_BASE);
        self.gs_base.set_encoding(HOST_STATE_GS_BASE);
        self.tr_base.set_encoding(HOST_STATE_TR_BASE);
        self.gdtr_base.set_encoding(HOST_STATE_GDTR_BASE);
        self.idtr_base.set_encoding(HOST_STATE_IDTR_BASE);
        self.ia32_sysenter_esp.set_encoding(HOST_STATE_IA32_SYSENTER_ESP);
        self.ia32_sysenter_eip.set_encoding(HOST_STATE_IA32_SYSENTER_EIP);
        self.rsp.set_encoding(HOST_STATE_RSP);
        self.rip.set_encoding(HOST_STATE_RIP);
    }

    fn commit(&mut self) {
        self.es.force_flush();
        self.cs.force_flush();
        self.ss.force_flush();
        self.ds.force_flush();
        self.fs.force_flush();
        self.gs.force_flush();
        self.tr.force_flush();

        let info = info_data();
        let exit = info.vm.vmcs.ctrl.exit.exit.field();

        if exit.load_ia32_pat() {
            self.ia32_pat.force_flush();
        }

        if exit.load_ia32_efer() {
            self.ia32_efer.force_flush();
        }

        if exit.load_ia32_perf() {
            self.ia32_perf.force_flush();
        }

        self.ia32_sysenter_cs.force_flush();

        self.cr0.force_flush();
        self.cr3.force_flush();
        self.cr4.force_flush();
        self.fs_base.force_flush();
        self.gs_base.force_flush();
        self.tr_base.force_flush();
        self.gdtr_base.force_flush();
        self.idtr_base.force_flush();
        self.ia32_sysenter_esp.force_flush();
        self.ia32_sysenter_eip.force_flush();
        self.rsp.force_flush();
        self.rip.force_flush();
    }
}

impl Setup for Guest {
    fn init(&mut self) {
        self.activity.set_field_value(ACTIVITY_STATE::Active as u64);
        self.vmcs_link_ptr.set_field_value(u64::max_value());
        // self.preempt_timer.set_field_value(1);

        let limit = rmode::ivt_limit(rmode::BIOS_MISC_INTERRUPT) as u64;
        self.idtr.limit.set_field_value(limit);

        self.ss.sel.set_field_value(rmode::BASE_SS);
        self.ss.base.set_field_value(rmode::BASE_SS*16);

        self.cs.limit.set_field_value(0xffff);
        self.ss.limit.set_field_value(0xffff);
        self.ds.limit.set_field_value(0xffff);
        self.es.limit.set_field_value(0xffff);
        self.fs.limit.set_field_value(0xffff);
        self.gs.limit.set_field_value(0xffff);

        self.cs.attr.set_field_value(SEG_ATTR_CODE_16_R0_CO as u64);
        self.ss.attr.set_field_value(SEG_ATTR_DATA_16_R0 as u64);
        self.ds.attr.set_field_value(SEG_ATTR_DATA_16_R3 as u64);
        self.es.attr.set_field_value(SEG_ATTR_DATA_16_R3 as u64);
        self.fs.attr.set_field_value(SEG_ATTR_DATA_16_R3 as u64);
        self.gs.attr.set_field_value(SEG_ATTR_DATA_16_R3 as u64);
        self.tr.attr.set_field_value(SEG_ATTR_TSS_32 as u64);
        self.ldtr.attr.set_field_value(SEG_ATTR_UNUSABLE as u64);

        #[cfg(feature = "debug_vmcs_setup")]
        log!("rdmsr IA32 PAT\n");
        self.ia32_pat.set_field_value(msr::rdmsr(msr::IA32_PAT));

        // XXX: TODO
        // self.ia32_dbgctl.set_field_value(msr::rdmsr(msr::IA32_DEBUGCTL));
        // self.ia32_bndcfg.set_field_value(msr::rdmsr(msr::IA32_BNDCFG));

        let rflags = self.rflags.field_mut();
        rflags.update_u64(0);
        rflags.set_it(true);

        self.rsp.set_field_value(rmode::BASE_SP);
        self.rip.set_field_value(rmode::BASE_IP);
    }

    // 16, 64, 32, Natural fields
    fn encode(&mut self) {
        self.es.sel.set_encoding(GUEST_STATE_ES_SEL);
        self.cs.sel.set_encoding(GUEST_STATE_CS_SEL);
        self.ss.sel.set_encoding(GUEST_STATE_SS_SEL);
        self.ds.sel.set_encoding(GUEST_STATE_DS_SEL);
        self.fs.sel.set_encoding(GUEST_STATE_FS_SEL);
        self.gs.sel.set_encoding(GUEST_STATE_GS_SEL);
        self.ldtr.sel.set_encoding(GUEST_STATE_LDTR_SEL);
        self.tr.sel.set_encoding(GUEST_STATE_TR_SEL);
        self.guest_intr.set_encoding(GUEST_STATE_INTR);
        self.pml_index.set_encoding(GUEST_STATE_PML_IDX);

        self.vmcs_link_ptr.set_encoding(GUEST_STATE_VMCS_LINK_PTR);
        self.ia32_dbgctl.set_encoding(GUEST_STATE_IA32_DBG_CTL);
        self.ia32_pat.set_encoding(GUEST_STATE_IA32_PAT);
        self.ia32_efer.set_encoding(GUEST_STATE_IA32_EFER);
        self.ia32_perf.set_encoding(GUEST_STATE_IA32_PERF_GLOBAL);
        self.ia32_bndcfg.set_encoding(GUEST_STATE_IA32_BNDCFG);
        self.pdpe_0.set_encoding(GUEST_STATE_PDPTE0);
        self.pdpe_1.set_encoding(GUEST_STATE_PDPTE1);
        self.pdpe_2.set_encoding(GUEST_STATE_PDPTE2);
        self.pdpe_3.set_encoding(GUEST_STATE_PDPTE3);

        self.es.limit.set_encoding(GUEST_STATE_ES_LIMIT);
        self.cs.limit.set_encoding(GUEST_STATE_CS_LIMIT);
        self.ss.limit.set_encoding(GUEST_STATE_SS_LIMIT);
        self.ds.limit.set_encoding(GUEST_STATE_DS_LIMIT);
        self.fs.limit.set_encoding(GUEST_STATE_FS_LIMIT);
        self.gs.limit.set_encoding(GUEST_STATE_GS_LIMIT);
        self.ldtr.limit.set_encoding(GUEST_STATE_LDTR_LIMIT);
        self.tr.limit.set_encoding(GUEST_STATE_TR_LIMIT);
        self.gdtr.limit.set_encoding(GUEST_STATE_GDTR_LIMIT);
        self.idtr.limit.set_encoding(GUEST_STATE_IDTR_LIMIT);
        self.es.attr.set_encoding(GUEST_STATE_ES_ACCESS_RIGHTS);
        self.cs.attr.set_encoding(GUEST_STATE_CS_ACCESS_RIGHTS);
        self.ss.attr.set_encoding(GUEST_STATE_SS_ACCESS_RIGHTS);
        self.ds.attr.set_encoding(GUEST_STATE_DS_ACCESS_RIGHTS);
        self.fs.attr.set_encoding(GUEST_STATE_FS_ACCESS_RIGHTS);
        self.gs.attr.set_encoding(GUEST_STATE_GS_ACCESS_RIGHTS);
        self.ldtr.attr.set_encoding(GUEST_STATE_LDTR_ACCESS_RIGHTS);
        self.tr.attr.set_encoding(GUEST_STATE_TR_ACCESS_RIGHTS);
        self.interrupt.set_encoding(GUEST_STATE_INT_STATE);
        self.activity.set_encoding(GUEST_STATE_ACTIVITY_STATE);
        self.smbase.set_encoding(GUEST_STATE_SMBASE);
        self.ia32_sysenter_cs.set_encoding(GUEST_STATE_IA32_SYSENTER_CS);
        self.preempt_timer.set_encoding(GUEST_STATE_PREEMPT_TIMER);

        self.cr0.set_encoding(GUEST_STATE_CR0);
        self.cr3.set_encoding(GUEST_STATE_CR3);
        self.cr4.set_encoding(GUEST_STATE_CR4);
        self.es.base.set_encoding(GUEST_STATE_ES_BASE);
        self.cs.base.set_encoding(GUEST_STATE_CS_BASE);
        self.ss.base.set_encoding(GUEST_STATE_SS_BASE);
        self.ds.base.set_encoding(GUEST_STATE_DS_BASE);
        self.fs.base.set_encoding(GUEST_STATE_FS_BASE);
        self.gs.base.set_encoding(GUEST_STATE_GS_BASE);
        self.tr.base.set_encoding(GUEST_STATE_TR_BASE);
        self.ldtr.base.set_encoding(GUEST_STATE_LDTR_BASE);
        self.gdtr.base.set_encoding(GUEST_STATE_GDTR_BASE);
        self.idtr.base.set_encoding(GUEST_STATE_IDTR_BASE);
        self.dr7.set_encoding(GUEST_STATE_DR7);
        self.rsp.set_encoding(GUEST_STATE_RSP);
        self.rip.set_encoding(GUEST_STATE_RIP);
        self.rflags.set_encoding(GUEST_STATE_RFLAGS);
        self.pending_dbg.set_encoding(GUEST_STATE_PENDING_DBG_EXCP);
        self.ia32_sysenter_esp.set_encoding(GUEST_STATE_IA32_SYSENTER_ESP);
        self.ia32_sysenter_eip.set_encoding(GUEST_STATE_IA32_SYSENTER_EIP);

        // Fake fields
        self.cr2.set_encoding(GUEST_STATE_CR2);
        self.dr6.set_encoding(GUEST_STATE_DR6);
    }

    fn commit(&mut self) {
        let info = info_data();        

        self.cr0.set_fixed(info.vmm.cpu.vmx.fixed.cr0);
        self.cr0.force_flush();
        #[cfg(feature = "debug_vmcs_setup")]
        log!("cr0\n1fix1 0b{:032b}\n0fix0 0b{:032b}\n      0b{:032b}\n",
             self.cr0.fixed().allow_0.0,
             self.cr0.fixed().allow_1.0,
             self.cr0.field().0
        );

        self.cr4.set_fixed(info.vmm.cpu.vmx.fixed.cr4);
        self.cr4.force_flush();
        #[cfg(feature = "debug_vmcs_setup")]
        log!("cr4\n1fix1 0b{:032b}\n0fix0 0b{:032b}\n      0b{:032b}\n",
             self.cr4.fixed().allow_0.0,
             self.cr4.fixed().allow_1.0,
             self.cr4.field().0
        );

        self.es.sel.force_flush();
        self.cs.sel.force_flush();
        self.ss.sel.force_flush();
        self.ds.sel.force_flush();
        self.fs.sel.force_flush();
        self.gs.sel.force_flush();
        self.ldtr.sel.force_flush();
        self.tr.sel.force_flush();
        // self.guest_intr.force_flush();
        // self.pml_index.force_flush();

        self.vmcs_link_ptr.force_flush();

        let entry = info.vm.vmcs.ctrl.entry.entry.field();

        if entry.load_ia32_pat() {
            self.ia32_pat.force_flush();
        }

        if entry.load_ia32_efer() {
            self.ia32_efer.force_flush();
        }

        if entry.load_ia32_perf() {
            self.ia32_perf.force_flush();
        }

        // self.ia32_dbgctl.force_flush();
        // self.ia32_bndcfg.force_flush();
        self.pdpe_0.force_flush();
        self.pdpe_1.force_flush();
        self.pdpe_2.force_flush();
        self.pdpe_3.force_flush();

        self.es.limit.force_flush();
        self.cs.limit.force_flush();
        self.ss.limit.force_flush();
        self.ds.limit.force_flush();
        self.fs.limit.force_flush();
        self.gs.limit.force_flush();
        self.ldtr.limit.force_flush();
        self.tr.limit.force_flush();
        self.gdtr.limit.force_flush();
        self.idtr.limit.force_flush();
        self.es.attr.force_flush();
        self.cs.attr.force_flush();
        self.ss.attr.force_flush();
        self.ds.attr.force_flush();
        self.fs.attr.force_flush();
        self.gs.attr.force_flush();
        self.ldtr.attr.force_flush();
        self.tr.attr.force_flush();
        self.interrupt.force_flush();
        self.activity.force_flush();
        self.smbase.force_flush();
        self.ia32_sysenter_cs.force_flush();

        if info.vmm.cpu.vmx.fixed.pin.allow_1.preempt() {
            self.preempt_timer.force_flush();
        }

        self.cr3.force_flush();
        self.es.base.force_flush();
        self.cs.base.force_flush();
        self.ss.base.force_flush();
        self.ds.base.force_flush();
        self.fs.base.force_flush();
        self.gs.base.force_flush();
        self.tr.base.force_flush();
        self.ldtr.base.force_flush();
        self.gdtr.base.force_flush();
        self.idtr.base.force_flush();
        self.dr7.force_flush();
        self.rsp.force_flush();
        self.rip.force_flush();
        self.rflags.force_flush();
        self.pending_dbg.force_flush();
        self.ia32_sysenter_esp.force_flush();
        self.ia32_sysenter_eip.force_flush();

        // Fake fields
        self.cr2.force_flush();
        self.dr6.force_flush();
    }
}
