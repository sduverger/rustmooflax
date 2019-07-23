use utils;

bitfield!{
    #[derive(Copy, Clone)]
    pub struct Rflags(u64);

    impl Debug;

    pub cf,set_cf:0;
    pub pf,set_pf: 2;
    pub af,set_af: 4;
    pub zf,set_zf: 6;
    pub sf,set_sf: 7;
    pub tf,set_tf: 8;
    pub it,set_it: 9;
    pub df,set_df: 10;
    pub of,set_of: 11;
    pub iopl,set_iopl: 13,12;
    pub nt,set_nt: 14;
    pub rf,set_rf: 16;
    pub vm,set_vm: 17;
    pub ac,set_ac: 18;
    pub vif,set_vif: 19;
    pub vip,set_vip: 20;
    pub id,set_id: 21;
}

impl Default for Rflags {
    fn default() -> Rflags {
        Rflags(1<<1)
    }
}

impl Rflags {
    fn filter(x: u64) -> u64 {
        let mbz: u64 = 1<<3 | 1<<5 | 1<<15 | !((1<<22) - 1);
        let mbo: u64 = 1<<1;
        (x & !mbz) | mbo
    }

    pub fn as_u16(&self) -> u16 { self.0 as u16}
}

impl utils::RawValue for Rflags {
    fn from_u32(x: u32) -> Rflags { Rflags(Self::filter(x as u64)) }
    fn as_u64(&self) -> u64 { self.0 }
    fn update_u64(&mut self, v: u64) { self.0 = Self::filter(v); }
}
