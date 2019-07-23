// Intel VMX registers
use cr;
use msr::*;
use utils;
use utils::RawValue;


// VMX fixed bits settings:
//
// allow_0 <=> if bit is 1, it is fixed to 1 in destination register
// allow_1 <=> if bit is 0, it is fixed to 0 in destination register
//
// consider allow_0 as fixed_1
//
// final = (wanted & allow_1) | allow_0
//
#[derive(Default, Copy, Clone)]
pub struct FixedReg<T> {
    pub allow_0: T, // fixed_1
    pub allow_1: T, // fixed_0
}

impl<T> FixedReg<T> where T: utils::RawValue {
    pub fn from_msr64(msr: u32) -> FixedReg<T> {
        let (a0,a1): (u32, u32);

        unsafe {
            asm!("rdmsr"
                 : "={eax}" (a0), "={edx}" (a1)
                 : "{ecx}" (msr)
                 : "memory": "volatile");
        }

        FixedReg {
            allow_0: T::from_u32(a0),
            allow_1: T::from_u32(a1),
        }
    }

    pub fn from_msr32(msr0: u32, msr1: u32) -> FixedReg<T> {
        let (a0,a1): (u32, u32);

        unsafe {
            asm!("rdmsr"
                 : "={eax}" (a0)
                 : "{ecx}" (msr0)
                 : "memory"
                 : "volatile");

            asm!("rdmsr"
                 : "={eax}" (a1)
                 : "{ecx}" (msr1)
                 : "memory"
                 : "volatile");
        }

        FixedReg {
            allow_0: T::from_u32(a0),
            allow_1: T::from_u32(a1),
        }
    }

    // pub fn mask_u64(&self, value: &T) -> u64 {
    pub fn mask_u64(&self, value: u64) -> u64 {
        (value & self.allow_1.as_u64()) | self.allow_0.as_u64()
    }
}

#[derive(Default, Copy, Clone)]
pub struct FixedRegisters {
    pub pin: FixedReg<ExecPinCtls>,
    pub proc1: FixedReg<ExecProc1Ctls>,
    pub proc2: FixedReg<ExecProc2Ctls>,
    pub entry: FixedReg<EntryCtls>,
    pub exit: FixedReg<ExitCtls>,
    pub cr0: FixedReg<cr::Cr0>,
    pub cr4: FixedReg<cr::Cr4>,
}


#[derive(Default, Copy, Clone)]
pub struct VMXInfo {
    pub basic: IA32VmxBasic,
    pub misc: IA32VmxMisc,
    pub ept: IA32VmxEptVpidCap,
    pub fixed: FixedRegisters,
}

impl VMXInfo {
    fn read_vmx_basic(&mut self) {
        self.basic = IA32VmxBasic( rdmsr(IA32_VMX_BASIC) );

        /* XXX: conditional compilation */
        if ! self.basic.io_insn() {
            panic!("vmx ins/outs info not given on VM-exit");
        }

        if self.basic.paddrw32() {
            panic!("vmx vmcs paddr width limited to 32 bits\n");
        }

        self.misc = IA32VmxMisc( rdmsr(IA32_VMX_MISC) );

        if ! self.misc.lma() {
            panic!("vmx misc ia32e/lma missing");
        }
    }

    fn read_vmx_fixed(&mut self) {
        if self.basic.true_f1() {
            self.fixed.pin   = FixedReg::from_msr64(
                IA32_VMX_TRUE_PINBASED_CTLS);
            self.fixed.proc1 = FixedReg::from_msr64(
                IA32_VMX_TRUE_PROCBASED_CTLS);
            self.fixed.entry = FixedReg::from_msr64(
                IA32_VMX_TRUE_ENTRY_CTLS);
            self.fixed.exit  = FixedReg::from_msr64(
                IA32_VMX_TRUE_EXIT_CTLS);
        } else {
            self.fixed.pin   = FixedReg::from_msr64(
                IA32_VMX_PINBASED_CTLS);
            self.fixed.proc1 = FixedReg::from_msr64(
                IA32_VMX_PROCBASED_CTLS);
            self.fixed.entry = FixedReg::from_msr64(
                IA32_VMX_ENTRY_CTLS);
            self.fixed.exit  = FixedReg::from_msr64(
                IA32_VMX_EXIT_CTLS);
        }

        if ! self.fixed.proc1.allow_1.proc2() {
            panic!("vmx missing secondary processor based");
        }

        self.fixed.proc2 = FixedReg::from_msr64(
            IA32_VMX_PROCBASED_CTLS2);

        if ! self.fixed.proc2.allow_1.uguest() {
            panic!("vmx missing unrestricted guest");
        }

        if ! self.fixed.proc2.allow_1.dt() {
            panic!("vmx desc table exiting not supported");
        }

        if ! self.fixed.proc2.allow_1.vpid() {
            panic!("vmx vpid not supported");
        }

        self.fixed.cr0 = FixedReg::from_msr32(
            IA32_VMX_CR0_FIXED0, IA32_VMX_CR0_FIXED1);

        // unrestricted guest
        self.fixed.cr0.allow_0.set_pe(false);
        self.fixed.cr0.allow_0.set_pg(false);

        self.fixed.cr0.allow_1.set_pe(true);
        self.fixed.cr0.allow_1.set_pg(true);


        self.fixed.cr4 = FixedReg::from_msr32(
            IA32_VMX_CR4_FIXED0, IA32_VMX_CR4_FIXED1);

        // #MCE
        self.fixed.cr4.allow_0.set_mce(true);
        self.fixed.cr4.allow_1.set_mce(true);
    }

    fn read_vmx_ept(&mut self) {
        if ! self.fixed.proc2.allow_1.ept() {
            panic!("vmx ept not supported");
        }

        self.ept = IA32VmxEptVpidCap(rdmsr(IA32_VMX_EPT_VPID_ENUM));

        if ! self.ept.wb() {
            panic!("vmx ept mem type only UC");
        }

        if ! self.ept.pwl4() {
            panic!("vmx ept unsupported page walk length");
        }

        if ! self.ept.invvpid() {
            panic!("vmx ept missing invvpid");
        }

        if ! self.ept.invept() {
            panic!("vmx ept missing invept");
        }
    }


    // Public API
    pub fn init(&mut self) {
        self.read_vmx_basic();
        self.read_vmx_fixed();
        self.read_vmx_ept();
    }
}



bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct ExecPinCtls(u32);

    impl Debug;

    pub eint,_:0;
    pub nmi,_:3;
    pub vnmi,_:5;
    pub preempt,_:6;
    pub pint,_:7;
}

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct ExecProc1Ctls(u32);

    impl Debug;

    pub iwe,_:2;
    pub tsc,set_tsc:3;
    pub hlt,_:7;
    pub invl,_:9;
    pub mwait,_:10;
    pub rdpmc,_:11;
    pub rdtsc,_:12;
    pub cr3l,set_cr3l:15;
    pub cr3s,_:16;
    pub cr8l,_:19;
    pub cr8s,_:20;
    pub tprs,_:21;
    pub nwe,_:22;
    pub mdr,_:23;
    pub ucio,_:24;
    pub usio,set_usio:25;
    pub mtf,_:27;
    pub umsr,set_umsr:28;
    pub mon,_:29;
    pub pause,_:30;
    pub proc2,set_proc2:31;
}

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct ExecProc2Ctls(u32);

    impl Debug;

    pub vapic,_:0;
    pub ept,set_ept:1;
    pub dt,set_dt:2;
    pub rdtscp,set_rdtscp:3;
    pub x2apic,_:4;
    pub vpid,set_vpid:5;
    pub wbinvd,_:6;
    pub uguest,set_uguest:7;
    pub reg,_:8;
    pub vintr,_:9;
    pub pause,_:10;
    pub rdrand,_:11;
    pub invpcid,_:12;
    pub vmfunc,_:13;
    pub shadow,_:14;
    pub encls,_:15;
    pub rdseed,_:16;
    pub pml,_:17;
    pub ve,_:18;
    pub ipt,_:19;
    pub xsave,_:20;
    pub tsc,_:25;
}

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct EntryCtls(u32);

    impl Debug;

    pub load_dbgctl,set_load_dbgctl:2;
    pub ia32e,_:9;
    pub smm,_:10;
    pub dual,_:11;
    pub load_ia32_perf,set_load_ia32_perf:13;
    pub load_ia32_pat,set_load_ia32_pat:14;
    pub load_ia32_efer,set_load_ia32_efer:15;
    pub load_ia32_bnd,set_load_ia32_bnd:16;
    pub ipt,_:17;
}

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct ExitCtls(u32);

    impl Debug;

    pub save_dbgctl,set_save_dbgctl:2;
    pub host_lmode,set_host_lmode:9;
    pub load_ia32_perf,set_load_ia32_perf:12;
    pub ack_int,set_ack_int:15;
    pub save_ia32_pat,set_save_ia32_pat:18;
    pub load_ia32_pat,set_load_ia32_pat:19;
    pub save_ia32_efer,set_save_ia32_efer:20;
    pub load_ia32_efer,set_load_ia32_efer:21;
    pub save_preempt_timer,_:22;
    pub clear_bnd,set_clear_bnd:23;
    pub ipt,_:24;
}

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct ExitReason(u32);

    impl Debug;

    pub u16, basic,_:15,0;
    pub enclave,_:27;
    pub mtf,_:28;
    pub root,_:29;
    pub entry,_:31;
}

#[derive(Debug,Copy,Clone)]
pub enum EventType {
    HardInt,   // External Interrupts
    Reserved,
    NMI,
    HardExcp,  // Exceptions but: int0, int3
    SoftInt,
    PSExcp,
    SoftExcp,  // int0, int1, int3
    Other,
}

use core::convert::TryFrom;
use self::EventType::*;
impl TryFrom<u8> for EventType {
    type Error = u8;
    fn try_from(kind: u8) -> Result<Self, Self::Error> {
        match kind {
            0 => Ok(HardInt),
            1 => Ok(Reserved),
            2 => Ok(NMI),
            3 => Ok(HardExcp),
            4 => Ok(SoftInt),
            5 => Ok(PSExcp),
            6 => Ok(SoftExcp),
            7 => Ok(Other),
            n => Err(n),
        }
    }
}

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct ExitInfoInterrupt(u32);

    impl Debug;

    pub u8, vector,_:7,0;
    pub u8, kind,_:10,8;
    pub v_err,_:11;
    pub nmi,_:12; // undefined for Exit Idt Vector, Entry Vector
    pub v,_:31;
}


// VMCS specific segment descriptor attributes pre-computed values
pub const SEG_ATTR_CODE_32_R0     : u32 = 0xc09b;
pub const SEG_ATTR_CODE_16_R0     : u32 = 0x809b;
pub const SEG_ATTR_CODE_16_R0_CO  : u32 = 0x809f;
pub const SEG_ATTR_CODE_16_R1_CO  : u32 = 0x80bf;
pub const SEG_ATTR_CODE_16_R3     : u32 = 0x80fb;
pub const SEG_ATTR_DATA_32_R0     : u32 = 0xc093;
pub const SEG_ATTR_DATA_16_R0     : u32 = 0x8093;
pub const SEG_ATTR_DATA_16_R1     : u32 = 0x80b3;
pub const SEG_ATTR_DATA_16_R3     : u32 = 0x80f3;
pub const SEG_ATTR_TSS_32         : u32 = 0x8b;
pub const SEG_ATTR_UNUSABLE       : u32 = 1<<16;


bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct SegAttr(u32);

    impl Debug;

    pub kind,_:3,0;
    pub s,_:4;
    pub dpl,_:6,5;
    pub p,_:7;
    pub l,_:13;
    pub d,_:14;
    pub g,_:15;
    pub u,_:16;
}



// XXX: macro to generate that impl
impl utils::RawValue for ExecPinCtls {
    fn from_u32(x: u32) -> ExecPinCtls { ExecPinCtls(x) }
    fn as_u64(&self) -> u64 { self.0 as u64 }
    fn update_u64(&mut self, v: u64) { self.0 = v as u32; }
}

impl utils::RawValue for ExecProc1Ctls {
    fn from_u32(x: u32) -> ExecProc1Ctls { ExecProc1Ctls(x) }
    fn as_u64(&self) -> u64 { self.0 as u64 }
    fn update_u64(&mut self, v: u64) { self.0 = v as u32; }
}
impl utils::RawValue for ExecProc2Ctls {
    fn from_u32(x: u32) -> ExecProc2Ctls { ExecProc2Ctls(x) }
    fn as_u64(&self) -> u64 { self.0 as u64 }
    fn update_u64(&mut self, v: u64) { self.0 = v as u32; }
}

impl utils::RawValue for EntryCtls {
    fn from_u32(x: u32) -> EntryCtls { EntryCtls(x) }
    fn as_u64(&self) -> u64 { self.0 as u64 }
    fn update_u64(&mut self, v: u64) { self.0 = v as u32; }
}

impl utils::RawValue for ExitCtls {
    fn from_u32(x: u32) -> ExitCtls { ExitCtls(x) }
    fn as_u64(&self) -> u64 { self.0 as u64 }
    fn update_u64(&mut self, v: u64) { self.0 = v as u32; }
}

impl utils::RawValue for ExitReason {
    fn from_u32(x: u32) -> ExitReason { ExitReason(x) }
    fn as_u64(&self) -> u64 { self.0 as u64 }
    fn update_u64(&mut self, v: u64) { self.0 = v as u32; }
}

impl utils::RawValue for ExitInfoInterrupt {
    fn from_u32(x: u32) -> ExitInfoInterrupt { ExitInfoInterrupt(x) }
    fn as_u64(&self) -> u64 { self.0 as u64 }
    fn update_u64(&mut self, v: u64) { self.0 = v as u32; }
}

impl utils::RawValue for SegAttr {
    fn from_u32(x: u32) -> SegAttr { SegAttr(x) }
    fn as_u64(&self) -> u64 { self.0 as u64 }
    fn update_u64(&mut self, v: u64) { self.0 = v as u32; }
}
