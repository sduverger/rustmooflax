use core;
use core::num::Wrapping;

pub fn aligned(addr: u64, sz: usize) -> bool {
    ((addr & ((sz - 1) as u64)) == 0)
}

pub fn align(addr: u64, sz: usize) -> u64 {
    (addr & ((!(sz - 1)) as u64) )
}

pub fn align_next(addr: u64, sz: usize) -> u64 {
    align(addr + (sz as u64), sz)
}

pub fn min<T>(a: T, b: T) -> T where T: core::cmp::PartialOrd {
    if a < b { a } else { b}
}

// Translate a memory address and size into an Option<slice>
pub unsafe fn addr_to_option_slice<'a>(paddr: u64, sz: usize) -> Option<&'a [u8]> {
    use core::mem;
    use core::slice;

    let ptr = mem::transmute(paddr);
    Some(slice::from_raw_parts(ptr, sz))
}



// XXX: create a real Raw type with set_u8,set_u16,set_u32 ... modular arith ...

pub trait RawValue {
    fn from_u32(x: u32) -> Self;
    fn as_u64(&self) -> u64;
    fn update_u64(&mut self, v: u64);
}


#[derive(Copy, Clone)]
pub struct Raw64(pub u64);

impl RawValue for Raw64 {
    fn from_u32(x: u32) -> Raw64 { Raw64(x as u64) }
    fn as_u64(&self) -> u64 { self.0 }
    fn update_u64(&mut self, v: u64) { self.0 = v; }
}

impl Raw64 {
    pub fn as_u32(&self) -> u32 { self.0 as u32 }
    pub fn as_u16(&self) -> u16 { self.0 as u16 }

    pub fn update_u32(&mut self, v: u32) {
        self.0 &= 0xffffffff00000000;
        self.0 |= v as u64;
    }
}

use core::fmt;

impl fmt::Debug for Raw64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Raw32(pub u32);

impl RawValue for Raw32 {
    fn from_u32(x: u32) -> Raw32 { Raw32(x) }
    fn as_u64(&self) -> u64 { self.0 as u64 }
    fn update_u64(&mut self, v: u64) { self.0 = v as u32; }
}

impl Raw32 {
    pub fn as_u16(&self) -> u16 { self.0 as u16 }
}



#[derive(Debug, Copy, Clone)]
pub struct Raw16(pub u16);

impl RawValue for Raw16 {
    fn from_u32(x: u32) -> Raw16 { Raw16(x as u16) }
    fn as_u64(&self) -> u64 { self.0 as u64 }
    fn update_u64(&mut self, v: u64) { self.0 = v as u16; }
}



pub trait ArithmeticMod16 where Self:RawValue {

    fn update(&mut self, left: u64, right: u16) {
        self.update_u64( (left & !0xffff) | (right as u64) );
    }

    fn sub_mod16(&mut self, value: u16) {
        let dst = self.as_u64();
        let res = (Wrapping(dst as u16) - Wrapping(value)).0;
        self.update(dst, res);
    }

    fn add_mod16(&mut self, value: u16) {
        let dst = self.as_u64();
        let res = (Wrapping(dst as u16) + Wrapping(value)).0;
        self.update(dst, res);

        // let low = (Wrapping(*left as u16) + Wrapping(right)).0;
        // *left = (*left & !0xffff) | (low as u64);
    }
}

impl ArithmeticMod16 for Raw64 {}
impl ArithmeticMod16 for Raw32 {}
