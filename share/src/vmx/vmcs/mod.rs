// Virtual Machine Control State
pub mod enc;
pub mod access;

use vmx::vmcs::access::*;
use vmx::regs::*;
use vmx::ept::*;

use cr;
use msr;
use utils;
use rflags::Rflags;
use paging::utils as pgutils;
use segmentation::SegmentSelector as SegSel;

#[repr(C, packed)]
pub struct Host {
    pub cr0: Field<cr::Cr0>,
    pub cr3: Field<cr::Cr3>,
    pub cr4: Field<cr::Cr4>,

    pub rsp: Field<utils::Raw64>,
    pub rip: Field<utils::Raw64>,

    pub cs: Field<SegSel>,
    pub ss: Field<SegSel>,
    pub ds: Field<SegSel>,
    pub es: Field<SegSel>,
    pub fs: Field<SegSel>,
    pub gs: Field<SegSel>,
    pub tr: Field<SegSel>,

    pub fs_base: Field<utils::Raw64>,
    pub gs_base: Field<utils::Raw64>,
    pub tr_base: Field<utils::Raw64>,
    pub gdtr_base: Field<utils::Raw64>,
    pub idtr_base: Field<utils::Raw64>,

    pub ia32_sysenter_cs: Field<utils::Raw32>,
    pub ia32_sysenter_esp: Field<utils::Raw64>,
    pub ia32_sysenter_eip: Field<utils::Raw64>,
    pub ia32_perf: Field<msr::IA32PerfGlobalCtl>,
    pub ia32_pat: Field<utils::Raw64>,
    pub ia32_efer: Field<msr::IA32Efer>,
}

#[repr(C, packed)]
pub struct GuestSegDesc {
    pub sel: Field<SegSel>,
    pub base: Field<utils::Raw64>,
    pub limit: Field<utils::Raw32>,
    pub attr: Field<SegAttr>,
}

#[repr(C, packed)]
pub struct GuestDTR {
    pub base: Field<utils::Raw64>,
    pub limit: Field<utils::Raw32>,
}

#[repr(C, packed)]
pub struct Guest {    
    pub cr0: FixedField<cr::Cr0>,
    pub cr2: FakeField<utils::Raw64>,
    pub cr3: Field<utils::Raw64>,
    pub cr4: FixedField<cr::Cr4>,
    pub dr6: FakeField<utils::Raw64>,
    pub dr7: Field<utils::Raw64>,

    pub rsp: Field<utils::Raw64>,
    pub rip: Field<utils::Raw64>,
    pub rflags: Field<Rflags>,

    pub es: GuestSegDesc,
    pub cs: GuestSegDesc,
    pub ss: GuestSegDesc,
    pub ds: GuestSegDesc,
    pub fs: GuestSegDesc,
    pub gs: GuestSegDesc,
    pub ldtr: GuestSegDesc,
    pub tr: GuestSegDesc,

    pub gdtr: GuestDTR,
    pub idtr: GuestDTR,

    pub ia32_dbgctl: Field<utils::Raw64>,
    pub ia32_sysenter_cs: Field<utils::Raw32>,
    pub ia32_sysenter_esp: Field<utils::Raw64>,
    pub ia32_sysenter_eip: Field<utils::Raw64>,
    pub ia32_perf: Field<msr::IA32PerfGlobalCtl>,
    pub ia32_pat: Field<utils::Raw64>,
    pub ia32_efer: Field<msr::IA32Efer>,
    pub ia32_bndcfg: Field<utils::Raw64>,

    pub smbase: Field<utils::Raw32>,

    // Non register
    pub activity: Field<utils::Raw32>,
    pub interrupt: Field<utils::Raw32>,
    pub pending_dbg: Field<utils::Raw64>,
    pub vmcs_link_ptr: Field<utils::Raw64>,
    pub preempt_timer: Field<utils::Raw32>,

    pub pdpe_0: Field<utils::Raw64>,
    pub pdpe_1: Field<utils::Raw64>,
    pub pdpe_2: Field<utils::Raw64>,
    pub pdpe_3: Field<utils::Raw64>,

    pub guest_intr: Field<utils::Raw16>,
    pub pml_index: Field<utils::Raw16>,
}

#[repr(C, packed)]
pub struct ExecCtl {
    pub pin: FixedField<ExecPinCtls>,
    pub proc1: FixedField<ExecProc1Ctls>,
    pub proc2: FixedField<ExecProc2Ctls>,

    pub excp_bitmap: Field<utils::Raw32>,
    pub pf_err_msk: Field<utils::Raw32>,
    pub pf_err_mch: Field<utils::Raw32>,

    // each bitmap is 4KB in size
    pub ioA_bitmap: Field<utils::Raw64>,
    pub ioB_bitmap: Field<utils::Raw64>,

    pub tsc_offset: Field<utils::Raw64>,

    pub cr0_mask: Field<cr::Cr0>,
    pub cr4_mask: Field<cr::Cr4>,

    pub cr0_read_shadow: Field<cr::Cr0>,
    pub cr4_read_shadow: Field<cr::Cr4>,

    pub cr3_target_0: Field<utils::Raw64>,
    pub cr3_target_1: Field<utils::Raw64>,
    pub cr3_target_2: Field<utils::Raw64>,
    pub cr3_target_3: Field<utils::Raw64>,
    pub cr3_target_cnt: Field<utils::Raw32>,

    pub apic_addr: Field<utils::Raw64>,
    pub vapic_addr: Field<utils::Raw64>,
    pub tpr_threshold: Field<utils::Raw64>,

    pub msr_bitmap: Field<utils::Raw64>,

    pub executive_vmcs_ptr: Field<utils::Raw64>,

    pub eptp: Field<EPTP>,
    pub vpid: Field<utils::Raw16>,

    pub ple_gap: Field<utils::Raw32>,
    pub ple_win: Field<utils::Raw32>,

    pub posted_int: Field<utils::Raw64>,
    pub vm_func: Field<utils::Raw64>,
    pub eptp_list: Field<utils::Raw64>,

    pub vmread_bitmap: Field<utils::Raw64>,
    pub vmwrite_bitmap: Field<utils::Raw64>,

    pub encls_bitmap: Field<utils::Raw64>,
    pub pml_addr: Field<utils::Raw64>,

    pub vmx_excp_addr: Field<utils::Raw64>,
    pub eptp_idx: Field<utils::Raw16>,

    pub xss_bitmap: Field<utils::Raw64>,
}

#[repr(C, packed)]
pub struct ExitCtl {
    pub exit: FixedField<ExitCtls>,

    pub msr_store_cnt: Field<utils::Raw32>,
    pub msr_store_addr: Field<utils::Raw64>,
    pub msr_load_cnt: Field<utils::Raw32>,
    pub msr_load_addr: Field<utils::Raw64>,
}

#[repr(C, packed)]
pub struct EntryCtl {
    pub entry: FixedField<EntryCtls>,

    pub msr_load_cnt: Field<utils::Raw32>,
    pub msr_load_addr: Field<utils::Raw64>,

    pub int_info: Field<ExitInfoInterrupt>,
    pub int_err_code: Field<utils::Raw32>,
    pub insn_len: Field<utils::Raw32>,
}


#[repr(C, packed)]
pub struct Exit {
    pub reason: Field<ExitReason>,
    pub qualification: Field<utils::Raw64>,

    pub guest_linear: Field<utils::Raw64>,
    pub guest_physical: Field<utils::Raw64>,

    pub int_info: Field<ExitInfoInterrupt>,
    pub int_err_code: Field<utils::Raw32>,

    pub idt_info: Field<ExitInfoInterrupt>,
    pub idt_err_code: Field<utils::Raw32>,

    pub insn_len: Field<utils::Raw32>,
    pub insn_info: Field<utils::Raw32>,

    pub io_rcx: Field<utils::Raw64>,
    pub io_rsi: Field<utils::Raw64>,
    pub io_rdi: Field<utils::Raw64>,
    pub io_rip: Field<utils::Raw64>,

    pub vmx_insn_err: Field<utils::Raw32>,
}

#[repr(C, packed)]
pub struct Ctl {
    pub exec: ExecCtl,
    pub exit: ExitCtl,
    pub entry: EntryCtl,
}

// Internal (not CPU) representation of a VMCS. We associate field
// encoding to their value resulting in a much bigger size than actual
// Intel hardware VMCS size. Should be only used as a cache to access
// hardware loaded VMCS.
#[repr(C, packed)]
pub struct VMCS {
    pub revision_id: u32,
    pub abort: u32,
    pub guest: Guest,
    pub host: Host,
    pub ctrl: Ctl,
    pub exit: Exit,
}

// 4KB aligned
pub struct Region([u8;pgutils::PG_4KB]);

impl Region {
    pub fn get_addr(&self) -> u64 { &self.0 as *const _ as u64 }

    pub fn set_revision_id(&mut self, revid: u32) {
        let ptr = &self.0 as *const _ as *mut u32;
        unsafe {*ptr = revid};
    }
}

// Memory area used by the CPU to handle VMM VMCS
#[repr(C, packed)]
pub struct VmmHardwareVMCS {
    pub region: Region,
}

// Generic bitmap handling
#[repr(C, packed)]
pub struct Bitmap([u8;pgutils::PG_4KB]);

impl Bitmap {
    pub fn get_addr(&self) -> u64 { &self.0 as *const _ as u64 }

    // XXX: implement IO/MSR bitmaps settings
    pub fn deny(&mut self, index: u32) {
        log!("IO/MSR bitmap deny no implemented !\n");
    }

    pub fn allow(&mut self, index: u32) {
        log!("IO/MSR bitmap allow no implemented !\n");
    }
}

#[repr(C, packed)]
pub struct CtlMSRAreaEntry {
    index: u32,
    rsv:   u32,
    data:  u64,
}

#[repr(C,packed)] // XXX: IA32VmxMisc.msr() for count
pub struct CtlMSRArea([CtlMSRAreaEntry;512]);

impl CtlMSRArea {
    pub fn get_addr(&self) -> u64 { &self.0 as *const _ as u64 }
}

// Memory area used by the CPU to handle VM VMCS and associated bitmaps
#[repr(C, packed)]
pub struct VmHardwareVMCS {
    // 4KB aligned
    pub region:  Region,
    pub ioA_map: Bitmap,
    pub ioB_map: Bitmap,
    pub msr_map: Bitmap,

    // 16 bytes aligned
    pub exit_store: CtlMSRArea,
    pub exit_load: CtlMSRArea,
    pub entry_load: CtlMSRArea,
}
