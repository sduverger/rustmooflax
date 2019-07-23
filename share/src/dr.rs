// Debug Registers

pub fn dr6_read() -> u64 {
    let ret: u64;
    unsafe { asm!("mov %dr6, $0" : "=r" (ret)) };
    ret
}

pub fn dr6_write(val: u64) {
    unsafe { asm!("mov $0, %dr6" :: "r" (val) : "memory") };
}
