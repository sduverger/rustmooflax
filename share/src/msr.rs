// Definition of various MSRs
use utils;
pub use x86_64::registers::msr::*;

// We use BitFlags wrapped into a module
// to have a namespace around bitfields
pub mod IA32_FEAT_CTL {
    bitflags! {
        pub struct Flags: u64 {
            const LOCK = 0b001;
            const VMX  = 0b100;
        }
    }
}

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct IA32VmxBasic(u64);

    impl Debug;

    pub u32, vmcs_rev_id,_:30,0;
    pub vmcs_size,_:44,32;
    pub paddrw32,_:48;
    pub smm,_:49;
    pub vmcs_type,_:53,50;
    pub io_insn,_:54;
    pub true_f1,_:55;
}

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct IA32VmxMisc(u64);

    impl Debug;

    pub preempt_rate,_:4,0;
    pub lma,_:5;
    pub hlt,_:6;
    pub sht,_:7;
    pub ipi,_:8;
    pub r2,_:15,9;
    pub cr3,_:24,16;
    pub msr,_:27,25;
    pub smm,_:28;
    pub r3,_:31,29;
    pub u32, mseg,_:63,32;
}

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct IA32VmxEptVpidCap(u64);

    impl Debug;

    pub xo,_:0;
    pub pwl4,_:6;
    pub uc,_:8;
    pub wb,_:14;
    pub pg_2m,_:16;
    pub pg_1g,_:17;
    pub invept,_:20;
    pub dirty,_:21;
    pub invept_s,_:25;
    pub invept_a,_:26;
    pub invvpid,_:32;
    pub invvpid_i,_:40;
    pub invvpid_s,_:41;
    pub invvpid_a,_:42;
    pub invvpid_r,_:43;
}

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct IA32PerfGlobalCtl(u64);

    impl Debug;

    pub pmc0,_:0;
    pub pmc1,_:1;

    pub fixed_ctr0,_:32;
    pub fixed_ctr1,_:33;
    pub fixed_ctr2,_:34;
}

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct IA32Efer(u64);

    impl Debug;

    pub syscall,_:0;
    pub ia32_e,_:8;
    pub ia32_a,_:10;
    pub nx_e,_:11;
}


// XXX: macro here

impl utils::RawValue for IA32PerfGlobalCtl {
    fn from_u32(x: u32) -> IA32PerfGlobalCtl { IA32PerfGlobalCtl(x as u64) }
    fn as_u64(&self) -> u64 { self.0 as u64 }
    fn update_u64(&mut self, v: u64) { self.0 = v; }
}

impl utils::RawValue for IA32Efer {
    fn from_u32(x: u32) -> IA32Efer { IA32Efer(x as u64) }
    fn as_u64(&self) -> u64 { self.0 as u64 }
    fn update_u64(&mut self, v: u64) { self.0 = v; }
}
