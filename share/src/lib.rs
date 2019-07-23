#![feature(lang_items, const_fn, asm, unique, try_from)]
#![no_std]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate bitfield;
extern crate spin;
extern crate x86; // cpuid
extern crate x86_64;
extern crate multiboot;
extern crate rlibc;

#[macro_use]
pub mod log;
pub mod uart;
pub mod utils;
pub mod paging;
pub mod smem;
pub mod vmm;
pub mod info;
pub mod gpr;
pub mod frame;
pub mod smap;
pub mod vmx;
pub mod msr;
pub mod mtrr;
pub mod cr;
pub mod dr;
pub mod rmode;
pub mod rflags;
pub mod exceptions;
pub mod segmentation;
pub mod interrupts;
pub mod cpu;
pub mod vm;
pub mod mmap;
pub mod pool;
