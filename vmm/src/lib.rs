#![feature(lang_items, const_fn, asm, unique, try_from)]
#![no_std]

use core::fmt;

#[macro_use]
extern crate bitfield;

#[macro_use]
extern crate share;

mod vmx;
mod interrupts;
mod disasm;
mod emulate;
mod cpumode;
mod vm;

// no explicit rust usage, so prevent LD gc-section
pub use vmx::exit::vmexit_handler;
pub use vmx::exit::vmresume_failure;
pub use interrupts::intr_hdlr;

use share::log;

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(args: fmt::Arguments, file: &'static str, line: u32) -> ! {
    log!("\n\n-= Panic in {}:{} =-\n\n", file, line);
    log::log_fmt(args);
    loop{}
}

#[lang = "eh_personality"]
extern fn eh_personality() {}
