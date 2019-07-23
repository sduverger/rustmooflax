#![feature(lang_items, const_fn, asm, unique, try_from)]
#![no_std]

extern crate rlibc;
extern crate x86_64;
extern crate multiboot;
extern crate spin;

use core::fmt;

#[macro_use]
extern crate share;

use share::log;
use share::log::Logger;
use share::uart::Serial;
use share::uart::SerialPort;
use share::gpr::GPR64Context;
use share::info::info_data;

mod smem;
mod segmentation;
mod interrupts;
mod vmem;
mod vmm;
mod vmx;
mod vm;
mod elf64;

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(args: fmt::Arguments, file: &'static str, line: u32) -> ! {
    log!("\n\n-= Panic in {}:{} =-\n\n", file, line);
    log::log_fmt(args);
    loop{}
}

#[lang = "eh_personality"]
extern fn eh_personality() {}

#[no_mangle]
pub extern fn init(mbi_addr: u64) -> &'static mut GPR64Context {
    Logger::init(Serial::init(SerialPort::Com1));
    log!("\n\n-= RustM00fl4x =-\n\n");

    smem::init(mbi_addr);
    segmentation::init();
    interrupts::init();
    vmem::init();

    vmm::init();
    vm::init();

    log!("== Starting VM ==\n");
    let info = info_data();
    info.vm.cpu.gpr
}

#[no_mangle]
pub extern fn vm_start_failure(perr: *mut u64) -> ! {
    let err = unsafe { *perr };
    log!("VM-Entry failure {}\n", err);
    loop{}
}
