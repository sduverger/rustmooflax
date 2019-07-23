
// Exceptions without error code
pub const DE: u32  =  0;
pub const DB: u32  =  1;
pub const NMI: u32 =  2;
pub const BP: u32  =  3;
pub const OF: u32  =  4;
pub const BR: u32  =  5;
pub const UD: u32  =  6;
pub const NM: u32  =  7;
pub const MO: u32  =  9;
pub const MF: u32  = 16;
pub const MC: u32  = 18;
pub const XF: u32  = 19;

// Exceptions with error code
pub const DF: u32  =  8;
pub const TS: u32  = 10;
pub const NP: u32  = 11;
pub const SS: u32  = 12;
pub const GP: u32  = 13;
pub const PF: u32  = 14;
pub const AC: u32  = 17;

#[derive(Debug, Copy, Clone)]
pub enum Exception {
    DivideZero,
    Debug,
    NonMaskable,
    Breakpoint,
    Overflow,
    Bound,
    InvalidOpCode,
    DeviceNotAvailable,
    DoubleFault,
    Coprocessor,
    InvalidTSS,
    SegmentNotPresent,
    StackFault,
    GeneralProtection,
    PageFault,
    Reserved,
    FloatingPoint,
    Alignment,
    MachineCheck,
    SIMD,
    Virtualization,
}    

use self::Exception::*;

impl Exception {
    pub fn has_code(&self) -> bool {
        match *self {
            DoubleFault       |
            InvalidTSS        |
            SegmentNotPresent |
            StackFault        |
            GeneralProtection |
            PageFault         |
            Alignment => true,
            _ => false,
        }
    }
}

use core::convert::TryFrom;
impl TryFrom<u8> for Exception {
    type Error = u8;
    fn try_from(vector: u8) -> Result<Self, Self::Error> {
        match vector {
             0 => Ok(DivideZero),
             1 => Ok(Debug),
             2 => Ok(NonMaskable),
             3 => Ok(Breakpoint),
             4 => Ok(Overflow),
             5 => Ok(Bound),
             6 => Ok(InvalidOpCode),
             7 => Ok(DeviceNotAvailable),
             8 => Ok(DoubleFault),
             9 => Ok(Coprocessor),
            10 => Ok(InvalidTSS),
            11 => Ok(SegmentNotPresent),
            12 => Ok(StackFault),
            13 => Ok(GeneralProtection),
            14 => Ok(PageFault),
            16 => Ok(FloatingPoint),
            17 => Ok(Alignment),
            18 => Ok(MachineCheck),
            19 => Ok(SIMD),
            20 => Ok(Virtualization),
            15|21...31 => Ok(Reserved),
            n => Err(n),
        }
    }
}
