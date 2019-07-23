use vmx::exit::VMMStatus;
use share::exceptions::Exception;
use share::vmx::regs::EventType;
use share::vmx::vmcs::access::Access;
use share::utils::RawValue;
use share::info::InformationData;
use emulate;
use cpumode::{CPUMode, CPUState};
use core;
use core::convert::TryFrom;

// #GP in real mode must be related to IDT vectoring events
// because of real mode IVT limit fixed to int15.
fn excp_gp_rmode(info: &mut InformationData) -> VMMStatus {
    let (vector, kind, valid) = {
        let idt_info = info.vm.vmcs.exit.idt_info.as_ref();
        (idt_info.vector(), idt_info.kind(), idt_info.v())
    };

    if !valid {
        #[cfg(feature = "debug_excp")]
        log!("rmode #GP not related to IDT event\n");
        VMMStatus::Fail
    } else {
        match EventType::try_from(kind) {
            Ok(EventType::SoftInt) => emulate::soft_int(info, vector),
            Ok(EventType::HardInt) => emulate::hard_int(info, vector),

            // XXX: this illustrates idiomatic Rust enum faillible From()
            // could be replaced by "_ =>" placeholder (as in handler() below)
            Ok(ev) => {
                log!("rmode #GP unsupported idt event {:?}\n", ev);
                VMMStatus::Fail
            },
            Err(value) => {
                log!("rmode #GP invalid idt event {}\n", value);
                VMMStatus::Fail
            },
        }
    }
}

fn excp_gp(info: &mut InformationData) -> VMMStatus {
    if CPUState::mode(info, CPUMode::real) {
        excp_gp_rmode(info)
    } else {
        VMMStatus::Fault
    }
}

pub fn handler(info: &mut InformationData) -> VMMStatus {
    let vector = info.vm.vmcs.exit.int_info.as_ref().vector();

    match Exception::try_from(vector) {
        Err(n) => {
            log!("invalid exception {}\n", n);
            VMMStatus::Fail
        },

        Ok(excp) => {
            #[cfg(feature = "debug_excp")]
            log!("Exception #{:#?}\n", excp);
            match excp {
                Exception::GeneralProtection => excp_gp(info),
                _ => {log!("-= unhandled =-"); VMMStatus::Fail},
            }
        },
    }
}
