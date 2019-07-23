// Definition of control registers
use utils;

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct Cr0(u64);

    impl Debug;

    pub pe,set_pe:0;
    pub mp,set_mp:1;
    pub em,set_em:2;
    pub ts,set_ts:3;
    pub et,set_et:4;
    pub ne,set_ne:5;
    pub wp,set_wp:16;
    pub am,set_am:18;
    pub nw,set_nw:29;
    pub cd,set_cd:30;
    pub pg,set_pg:31;
}

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct Cr3(u64);

    impl Debug;

    pub pwt,_:3;
    pub pcd,_:4;
    pub addr,_:51,12;
}

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct Cr4(u64);

    impl Debug;

    pub vme,set_vme:0;
    pub pvi,set_pvi:1;
    pub tsd,set_tsd:2;
    pub de,set_de:3;
    pub pse,set_pse:4;
    pub pae,set_pae:5;
    pub mce,set_mce:6;
    pub pge,set_pge:7;
    pub pce,set_pce:8;
    pub osfxsr,set_osfxsr:9;
    pub osxmmexcpt,set_osxmmexcpt:10;
    pub umip,set_umip:11;
    pub vmxe,set_vmxe:13;
    pub smxe,set_smxe:14;
    pub fsgsbase,set_fsgsbase:16;
    pub pcide,set_pcide:17;
    pub osxsave,set_osxsave:18;
    pub smep,set_smep:20;
    pub smap,set_smap:21;
    pub pke,set_pke:22;
}

pub fn cr2_write(val: u64) {
    unsafe { asm!("mov $0, %cr2" :: "r" (val) : "memory") };
}

// XXX: macro to generate that impl
impl utils::RawValue for Cr0 {
    fn from_u32(x: u32) -> Cr0 { Cr0(x as u64) }
    fn as_u64(&self) -> u64 { self.0 }
    fn update_u64(&mut self, v: u64) { self.0 = v; }
}

impl utils::RawValue for Cr3 {
    fn from_u32(x: u32) -> Cr3 { Cr3(x as u64) }
    fn as_u64(&self) -> u64 { self.0 }
    fn update_u64(&mut self, v: u64) { self.0 = v; }
}

impl utils::RawValue for Cr4 {
    fn from_u32(x: u32) -> Cr4 { Cr4(x as u64) }
    fn as_u64(&self) -> u64 { self.0 }
    fn update_u64(&mut self, v: u64) { self.0 = v; }
}
