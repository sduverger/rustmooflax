.text

.globl __vmx_vmwrite
.type  __vmx_vmwrite,"function"

.globl __vmx_vmread
.type  __vmx_vmread,"function"

.globl __vmx_vmclear
.type  __vmx_vmclear,"function"

.globl __vmx_vmload
.type  __vmx_vmload,"function"

.globl __vmx_vmxon
.type  __vmx_vmxon,"function"

.globl vmx_vmlaunch
.type  vmx_vmlaunch,"function"

        
/*
** VM-entry
*/
vmx_vmlaunch:
        popq    %r15
        popq    %r14
        popq    %r13
        popq    %r12
        popq    %r11
        popq    %r10
        popq    %r9
        popq    %r8
        popq    %rdi
        popq    %rsi
        popq    %rbp
        popq    %rbx
        popq    %rdx
        popq    %rcx
        popq    %rax
        vmlaunch

/*
** VM-entry failure
**
** rsp + 16 [  vmx_err     ]
** rsp +  8 [  fake_ret @  ]
** rsp +  0 [  vm_fail @   ]
**
** Params:
**      RDI = mem64 VMX error code ptr = @vmx_err
*/
vmx_vmlaunch_failure:
        mov     %rsp, %rdi
        push    $0xbadc0de
        push    $vm_start_failure
        jmp     vmx_check_error

/*
** Enter VMX root operations
**
** params:
**      RDI = mem64 VMXON region paddr
**
** returns:
**      0 on failure
**      1 on success
*/
__vmx_vmxon:
        vmxon   (%rdi)
        jc      vmx_fail
        jmp     vmx_success

/*
** Clear VMCS
**
** params:
**      RDI = mem64 VMX error code ptr
**      RSI = mem64 VMCS region paddr
**
** returns:
**      0 on failure
**      1 on success
*/
__vmx_vmclear:
        vmclear (%rsi)
        jmp     vmx_check_error

/*
** Load VMCS
**
** params:
**      RDI = mem64 VMX error code ptr
**      RSI = mem64 VMCS region paddr
**
** returns:
**      0 on failure
**      1 on success
*/
__vmx_vmload:
        vmptrld (%rsi)
        jmp     vmx_check_error

/*
** VM write
**
** params:
**      RDI = mem64 VMX error code ptr
**      RSI = value to write
**      RDX = VMCS field encoding
**
** returns:
**      0 on failure
**      1 on success
*/
__vmx_vmwrite:
        vmwrite %rsi, %rdx
        jmp     vmx_check_error

/*
** VM read
**
** params:
**      RDI = mem64 VMX error code ptr
**      RSI = mem64 read value ptr
**      RDX = VMCS field encoding
**
** returns:
**      0 on failure
**      1 on success
*/
__vmx_vmread:
        vmread  %rdx, (%rsi)
        jmp     vmx_check_error

/*
** VMX insn error checks
*/
vmx_check_error:
        jz      vmx_fail_valid
        jc      vmx_fail_invalid

vmx_success:
        mov     $1, %rax
        ret

/*
** VM Fail Valid : ZF=1
**
** read VMCS instruction error (0x4400)
** store it to (%rdi)
*/
vmx_fail_valid:
        push    %rdx
        mov     $0x4400, %rdx
        vmread  %rdx, (%rdi)
        pop     %rdx
        jmp     vmx_fail

/*
** VM Fail Invalid : CF=1
**
** VMCS instruction error code is 0
*/
vmx_fail_invalid:
        movl    $0, (%rdi)

vmx_fail:
        xor     %rax, %rax
        ret
