use share::vmx::vmcs::*;
use share::vmx::vmcs::access::Access;
use share::info::info_data;

pub trait Commit {
    // Every VMCS sub structures should implement this trait
    fn commit(&mut self) {}
}

impl Commit for VMCS {
    fn commit(&mut self) {
        // clear read/dirty
        self.exit.commit();

        // actually commit values if needed
        self.ctrl.commit();
        self.guest.commit();
        self.host.commit();
    }
}

impl Commit for Exit {
    fn commit(&mut self) {
        self.guest_physical.clear();
        self.vmx_insn_err.clear();
        self.reason.clear();
        self.int_info.clear();
        self.int_err_code.clear();
        self.idt_info.clear();
        self.idt_err_code.clear();
        self.insn_len.clear();
        self.insn_info.clear();
        self.qualification.clear();
        self.io_rcx.clear();
        self.io_rsi.clear();
        self.io_rdi.clear();
        self.io_rip.clear();
        self.guest_linear.clear();
    }
}

impl Commit for Ctl {
    fn commit(&mut self) {
        self.entry.commit();
        self.exec.commit();
        self.exit.commit();
    }
}

impl Commit for EntryCtl {
    fn commit(&mut self) {
        self.msr_load_addr.flush();
        self.entry.flush();
        self.msr_load_cnt.flush();
        self.int_info.flush();
        self.int_err_code.flush();
        self.insn_len.flush();
    }
}

impl Commit for ExecCtl {
    fn commit(&mut self) {
        let info = info_data();

        self.pin.flush();
        self.proc1.flush();
        self.proc2.flush();
        self.vpid.flush();
        // self.eptp_idx.flush();

        self.ioA_bitmap.flush();
        self.ioB_bitmap.flush();
        self.msr_bitmap.flush();
        self.executive_vmcs_ptr.flush();
        self.tsc_offset.flush();
        self.eptp.flush();

        if info.vmm.cpu.vmx.fixed.proc1.allow_1.tprs() {
            self.vapic_addr.flush();
        }

        if info.vmm.cpu.vmx.fixed.proc2.allow_1.vapic() {
            self.apic_addr.flush();
        }

        // self.pml_addr.flush();
        // self.posted_int.flush();
        // self.vm_func.flush();
        // self.eptp_list.flush();
        // self.vmread_bitmap.flush();
        // self.vmwrite_bitmap.flush();
        // self.vmx_excp_addr.flush();
        // self.xss_bitmap.flush();
        // self.encls_bitmap.flush();

        self.excp_bitmap.flush();
        self.pf_err_msk.flush();
        self.pf_err_mch.flush();
        self.cr3_target_cnt.flush();
        self.tpr_threshold.flush();
        // self.ple_gap.flush();
        // self.ple_win.flush();

        self.cr0_mask.flush();
        self.cr4_mask.flush();
        self.cr0_read_shadow.flush();
        self.cr4_read_shadow.flush();
        self.cr3_target_0.flush();
        self.cr3_target_1.flush();
        self.cr3_target_2.flush();
        self.cr3_target_3.flush();
    }
}

impl Commit for  ExitCtl {
    fn commit(&mut self) {
        self.msr_store_addr.flush();
        self.msr_load_addr.flush();

        let info = info_data();
        self.exit.flush();
        self.msr_store_cnt.flush();
        self.msr_load_cnt.flush();
    }
}

impl Commit for Guest {
    fn commit(&mut self) {
        let info = info_data();        

        self.cr0.flush();
        self.cr4.flush();
        self.es.sel.flush();
        self.cs.sel.flush();
        self.ss.sel.flush();
        self.ds.sel.flush();
        self.fs.sel.flush();
        self.gs.sel.flush();
        self.ldtr.sel.flush();
        self.tr.sel.flush();
        // self.guest_intr.flush();
        // self.pml_index.flush();

        self.vmcs_link_ptr.flush();

        let entry = info.vm.vmcs.ctrl.entry.entry.field();

        if entry.load_ia32_pat() {
            self.ia32_pat.flush();
        }

        if entry.load_ia32_efer() {
            self.ia32_efer.flush();
        }

        if entry.load_ia32_perf() {
            self.ia32_perf.flush();
        }

        // self.ia32_dbgctl.flush();
        // self.ia32_bndcfg.flush();
        self.pdpe_0.flush();
        self.pdpe_1.flush();
        self.pdpe_2.flush();
        self.pdpe_3.flush();

        self.es.limit.flush();
        self.cs.limit.flush();
        self.ss.limit.flush();
        self.ds.limit.flush();
        self.fs.limit.flush();
        self.gs.limit.flush();
        self.ldtr.limit.flush();
        self.tr.limit.flush();
        self.gdtr.limit.flush();
        self.idtr.limit.flush();
        self.es.attr.flush();
        self.cs.attr.flush();
        self.ss.attr.flush();
        self.ds.attr.flush();
        self.fs.attr.flush();
        self.gs.attr.flush();
        self.ldtr.attr.flush();
        self.tr.attr.flush();
        self.interrupt.flush();
        self.activity.flush();
        self.smbase.flush();
        self.ia32_sysenter_cs.flush();

        if info.vmm.cpu.vmx.fixed.pin.allow_1.preempt() {
            self.preempt_timer.flush();
        }

        self.cr3.flush();
        self.es.base.flush();
        self.cs.base.flush();
        self.ss.base.flush();
        self.ds.base.flush();
        self.fs.base.flush();
        self.gs.base.flush();
        self.tr.base.flush();
        self.ldtr.base.flush();
        self.gdtr.base.flush();
        self.idtr.base.flush();
        self.dr7.flush();
        self.rsp.flush();
        self.rip.flush();
        self.rflags.flush();
        self.pending_dbg.flush();
        self.ia32_sysenter_esp.flush();
        self.ia32_sysenter_eip.flush();

        // Fake fields
        self.cr2.flush();
        self.dr6.flush();
    }
}

impl Commit for Host {
    fn commit(&mut self) {
        self.es.flush();
        self.cs.flush();
        self.ss.flush();
        self.ds.flush();
        self.fs.flush();
        self.gs.flush();
        self.tr.flush();

        let info = info_data();
        let exit = info.vm.vmcs.ctrl.exit.exit.field();

        if exit.load_ia32_pat() {
            self.ia32_pat.flush();
        }

        if exit.load_ia32_efer() {
            self.ia32_efer.flush();
        }

        if exit.load_ia32_perf() {
            self.ia32_perf.flush();
        }

        self.ia32_sysenter_cs.flush();

        self.cr0.flush();
        self.cr3.flush();
        self.cr4.flush();
        self.fs_base.flush();
        self.gs_base.flush();
        self.tr_base.flush();
        self.gdtr_base.flush();
        self.idtr_base.flush();
        self.ia32_sysenter_esp.flush();
        self.ia32_sysenter_eip.flush();
        self.rsp.flush();
        self.rip.flush();
    }
}
