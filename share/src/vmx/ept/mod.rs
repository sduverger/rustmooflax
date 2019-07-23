pub mod map;

use utils;
use utils::RawValue;
use info::InformationData;

// VPID Invalidation
pub enum VPID_INV_TYPE {
    Addr = 0,
    SingleAll = 1,
    All = 2,
    Single = 3,
}

impl Default for VPID_INV_TYPE {
    fn default() -> VPID_INV_TYPE {
        VPID_INV_TYPE::All
    }
}


// privilege
pub const PVL_R:   u64 = 1;
pub const PVL_W:   u64 = 2;
pub const PVL_X:   u64 = 4;
pub const PVL_RWX: u64 = PVL_R | PVL_W | PVL_X;

// memory type
pub const MMT_UC: u64 = 0;
pub const MMT_WC: u64 = 1;
pub const MMT_WT: u64 = 4;
pub const MMT_WP: u64 = 5;
pub const MMT_WB: u64 = 6;

pub const fn attr_pvl_msk() -> u64 { 7    }
pub const fn attr_mmt_msk() -> u64 { 7<<3 }
pub const fn attr_pvl_dft() -> u64 { PVL_RWX }

pub fn attr_dft(info: &InformationData) -> u64 {
    let pvl = attr_pvl_dft();
    let cch = if info.vmm.cpu.mtrr.def.e() {
        info.vmm.cpu.mtrr.def.kind()
    } else {
        MMT_UC
    };
    (cch<<3 | pvl)
}


bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct EPTP(u64);

    impl Debug;

    pub cache,set_cache:2,0;
    pub u8, pwl,set_pwl:5,3;
    pub acc_dirty,_:1;
    pub addr,set_addr:51,12;
}

impl utils::RawValue for EPTP {
    fn from_u32(x: u32) -> EPTP { EPTP(x as u64) }
    fn as_u64(&self) -> u64 { self.0 }
    fn update_u64(&mut self, v: u64) { self.0 = v; }
}
