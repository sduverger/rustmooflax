use vmx::exit::VMMStatus;
use share::info::InformationData;
use cpumode::{CPUMode, CPUState};

pub mod rmode;

pub fn soft_int(info: &mut InformationData, vector: u8) -> VMMStatus {
    // int3/into area 1 byte long and raise SoftExcp not SoftInt
    interrupt(info, vector, 2)
}

pub fn hard_int(info: &mut InformationData, vector: u8) -> VMMStatus {
    interrupt(info, vector, 0)
}

fn interrupt(info: &mut InformationData, vector: u8, isz: u16) -> VMMStatus {
    if CPUState::mode(info, CPUMode::real) {
        rmode::interrupt(info, vector, isz)
    } else {
        VMMStatus::Fail
    }
}
