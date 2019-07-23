use vmx;
use vmx::exit::VMMStatus;
use share::info::InformationData;

#[derive(Debug, Copy, Clone)]
pub enum BasicReason {
    ExceptionOrNMI,
    ExternalInterrupt,
    TripleFault,
    INIT,
    SIPI,
    IOSMI,
    OtherSMI,
    InterruptWindows,
    NMIWindow,
    TaskSwitch,
    CPUID,
    GETSEC,
    HLT,
    INVD,
    INVLPG,
    RDPMC,
    RDTSC,
    RSM,
    VMCALL,
    VMCLEAR,
    VMLAUNCH,
    VMPTRLD,
    VMPTRST,
    VMREAD,
    VMRESUME,
    VMWRITE,
    VMXOFF,
    VMXON,
    CRAccess,
    DRAcess,
    IO,
    RDMSR,
    WRMSR,
    InvalidGuestState,
    MSRLoading,
    MWAIT,
    MTF,
    MONITOR,
    PAUSE,
    MachineCheckEvent,
    TPR,
    APICAccess,
    VEOI,
    GDTR,
    LDTR,
    EPTViolation,
    EPTMisconfig,
    INVEPT,
    RDTSCP,
    PreemptTimer,
    INVVPID,
    WBINVD,
    XSETBV,
    APICWrite,
    RDRAND,
    INVPCID,
    VMFUNC,
    ENCLS,
    RDSEED,
    PageModLogFull,
    XSAVES,
    XRSTORS,
}

use self::BasicReason::*;
use core::convert::TryFrom;

impl TryFrom<u16> for BasicReason {
    type Error = u16;
    fn try_from(basic: u16) -> Result<Self, Self::Error> {
        match basic {
            0  => Ok(ExceptionOrNMI),
            1  => Ok(ExternalInterrupt),
            2  => Ok(TripleFault),
            3  => Ok(INIT),
            4  => Ok(SIPI),
            5  => Ok(IOSMI),
            6  => Ok(OtherSMI),
            7  => Ok(InterruptWindows),
            8  => Ok(NMIWindow),
            9  => Ok(TaskSwitch),
            10 => Ok(CPUID),
            11 => Ok(GETSEC),
            12 => Ok(HLT),
            13 => Ok(INVD),
            14 => Ok(INVLPG),
            15 => Ok(RDPMC),
            16 => Ok(RDTSC),
            17 => Ok(RSM),
            18 => Ok(VMCALL),
            19 => Ok(VMCLEAR),
            20 => Ok(VMLAUNCH),
            21 => Ok(VMPTRLD),
            22 => Ok(VMPTRST),
            23 => Ok(VMREAD),
            24 => Ok(VMRESUME),
            25 => Ok(VMWRITE),
            26 => Ok(VMXOFF),
            27 => Ok(VMXON),
            28 => Ok(CRAccess),
            29 => Ok(DRAcess),
            30 => Ok(IO),
            31 => Ok(RDMSR),
            32 => Ok(WRMSR),
            33 => Ok(InvalidGuestState),
            34 => Ok(MSRLoading),
            36 => Ok(MWAIT),
            37 => Ok(MTF),
            39 => Ok(MONITOR),
            40 => Ok(PAUSE),
            41 => Ok(MachineCheckEvent),
            43 => Ok(TPR),
            44 => Ok(APICAccess),
            45 => Ok(VEOI),
            46 => Ok(GDTR),
            47 => Ok(LDTR),
            48 => Ok(EPTViolation),
            49 => Ok(EPTMisconfig),
            50 => Ok(INVEPT),
            51 => Ok(RDTSCP),
            52 => Ok(PreemptTimer),
            53 => Ok(INVVPID),
            54 => Ok(WBINVD),
            55 => Ok(XSETBV),
            56 => Ok(APICWrite),
            57 => Ok(RDRAND),
            58 => Ok(INVPCID),
            59 => Ok(VMFUNC),
            60 => Ok(ENCLS),
            61 => Ok(RDSEED),
            62 => Ok(PageModLogFull),
            63 => Ok(XSAVES),
            64 => Ok(XRSTORS),

            n  => Err(n),
        }
    }
}

impl BasicReason {
    pub fn resolve(info: &mut InformationData, basic: u16) -> VMMStatus {
        match BasicReason::try_from(basic) {
            // XXX: fast match without sub-match reason
            // Ok(ExceptionOrNMI) => vmx::exit::excp::handler(info),
            // Ok(reason) => {log!("unhandled {:?}\n", reason);VMMStatus::Fail},

            Err(n)     => {
                log!("invalid reason {}", n);
                VMMStatus::Fail
            },

            Ok(reason) => {
                #[cfg(feature = "debug_reason")]
                log!("vm-exit {:?}\n", reason);
                match reason {
                    ExceptionOrNMI => vmx::exit::excp::handler(info),
                    _ => {log!("-= unhandled =-\n"); VMMStatus::Fail},
                }
            },
        }
    }
}

