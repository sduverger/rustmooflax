// Low level VMX instructions
extern {
    fn __vmx_vmxon(vmcs: *const u64) -> u8;
    fn __vmx_vmclear(err: *mut u64, vmcs: *const u64) -> u8;
    fn __vmx_vmload(err: *mut u64, vmcs: *const u64) -> u8;
    fn __vmx_vmread(err: *mut u64, val: *mut u64, enc: u64) -> u8;
    fn __vmx_vmwrite(err: *mut u64, val: u64, enc: u64) -> u8;
}

pub fn vmxon(vmcs: u64) {
    let pvmcs = &vmcs as *const _;

    log!("vmx::vmxon(0x{:x})\n", vmcs);

    if unsafe { __vmx_vmxon(pvmcs) } == 0 {
        panic!("vmxon(0x{:x})", vmcs);
    }
}

pub fn vmclear(vmcs: u64) {
    let mut err: u64 = 0;
    let perr = &mut err as *mut _;
    let pvmcs = &vmcs as *const _;

    log!("vmx::vmclear(0x{:x})\n", vmcs);

    if unsafe { __vmx_vmclear(perr, pvmcs) } == 0 {
        panic!("vmclear(0x{:x}) err {}", vmcs, err);
    }
}

pub fn vmload(vmcs: u64) {
    let mut err: u64 = 0;
    let perr = &mut err as *mut _;
    let pvmcs = &vmcs as *const _;

    log!("vmx::vmload(0x{:x})\n", vmcs);

    if unsafe { __vmx_vmload(perr, pvmcs) } == 0 {
        panic!("vmload(0x{:x}) err {}", vmcs, err);
    }
}

// XXX: implement in vmcs as VMAccess methods ?
pub fn vmread(enc: u64) -> u64 {
    let mut err: u64 = 0;
    let mut val: u64 = 0;
    let perr = &mut err as *mut _;
    let pval = &mut val as *mut _;

    if unsafe { __vmx_vmread(perr, pval, enc) } == 0 {
        panic!("vmread(0x{:x}) err {}", enc, err);
    }

    #[cfg(feature = "debug_vmread")]
    log!("vmx::vmread(0x{:x}) 0x{:x}\n", enc, val);

    val
}

pub fn vmwrite(val: u64, enc: u64) {
    let mut err: u64 = 0;
    let perr = &mut err as *mut _;

    #[cfg(feature = "debug_vmwrite")]
    log!("vmx::vmwrite(0x{:x}, 0x{:x})\n", enc, val);

    if unsafe { __vmx_vmwrite(perr, val, enc) } == 0 {
        panic!("vmwrite(0x{:x}, 0x{:x}) err {}", enc, val, err);
    }
}
