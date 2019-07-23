// VMX VMCS fields encoding
pub const ACCESS_FULL:                    u32 = 0;
pub const ACCESS_HIGH:                    u32 = 1;

pub const TYPE_CTL:                       u32 = 0;
pub const TYPE_RO:                        u32 = 1;
pub const TYPE_G_STATE:                   u32 = 2;
pub const TYPE_H_STATE:                   u32 = 3;

pub const WIDTH_16:                       u32 = 0;
pub const WIDTH_64:                       u32 = 1;
pub const WIDTH_32:                       u32 = 2;
pub const WIDTH_NAT:                      u32 = 3;

// Fake fields
pub const GUEST_STATE_CR2:                u32 = 0;
pub const GUEST_STATE_DR6:                u32 = 1;

// 16-bit fields
pub const EXEC_CTRL_VPID:                 u32 = 0;
pub const EXEC_CTRL_EPTP_IDX:             u32 = 4;

pub const GUEST_STATE_ES_SEL:             u32 = 0x800;
pub const GUEST_STATE_CS_SEL:             u32 = 0x802;
pub const GUEST_STATE_SS_SEL:             u32 = 0x804;
pub const GUEST_STATE_DS_SEL:             u32 = 0x806;
pub const GUEST_STATE_FS_SEL:             u32 = 0x808;
pub const GUEST_STATE_GS_SEL:             u32 = 0x80a;
pub const GUEST_STATE_LDTR_SEL:           u32 = 0x80c;
pub const GUEST_STATE_TR_SEL:             u32 = 0x80e;
pub const GUEST_STATE_INTR:               u32 = 0x810;
pub const GUEST_STATE_PML_IDX:            u32 = 0x812;

pub const HOST_STATE_ES_SEL:              u32 = 0xc00;
pub const HOST_STATE_CS_SEL:              u32 = 0xc02;
pub const HOST_STATE_SS_SEL:              u32 = 0xc04;
pub const HOST_STATE_DS_SEL:              u32 = 0xc06;
pub const HOST_STATE_FS_SEL:              u32 = 0xc08;
pub const HOST_STATE_GS_SEL:              u32 = 0xc0a;
pub const HOST_STATE_TR_SEL:              u32 = 0xc0c;

// 64-bit fields
pub const EXEC_CTRL_ADDR_IO_MAP_A:        u32 = 0x2000;
pub const EXEC_CTRL_ADDR_IO_MAP_B:        u32 = 0x2002;
pub const EXEC_CTRL_ADDR_MSR_MAP:         u32 = 0x2004;
pub const EXIT_CTRL_MSR_STORE_ADDR:       u32 = 0x2006;
pub const EXIT_CTRL_MSR_LOAD_ADDR:        u32 = 0x2008;
pub const ENTRY_CTRL_MSR_LOAD_ADDR:       u32 = 0x200a;
pub const EXEC_CTRL_VMCS_PTR:             u32 = 0x200c;
pub const EXEC_CTRL_PML_ADDR:             u32 = 0x200e;
pub const EXEC_CTRL_TSC_OFFSET:           u32 = 0x2010;
pub const EXEC_CTRL_VAPIC_PAGE_ADDR:      u32 = 0x2012;
pub const EXEC_CTRL_APIC_PAGE_ADDR:       u32 = 0x2014;
pub const EXEC_CTRL_POSTED_INT:           u32 = 0x2016;
pub const EXEC_CTRL_VM_FUNC:              u32 = 0x2018;
pub const EXEC_CTRL_EPTP:                 u32 = 0x201a;
pub const EXEC_CTRL_EPTP_LIST:            u32 = 0x2024;
pub const EXEC_CTRL_VMREAD_MAP:           u32 = 0x2026;
pub const EXEC_CTRL_VMWRITE_MAP:          u32 = 0x2028;
pub const EXEC_CTRL_VMX_EXCP_ADDR:        u32 = 0x202a;
pub const EXEC_CTRL_XSS_MAP:              u32 = 0x202c;
pub const EXEC_CTRL_ENCLS_MAP:            u32 = 0x202e;

pub const EXIT_INFO_GUEST_PHYSICAL_ADDR:  u32 = 0x2400;

pub const GUEST_STATE_VMCS_LINK_PTR:      u32 = 0x2800;
pub const GUEST_STATE_IA32_DBG_CTL:       u32 = 0x2802;
pub const GUEST_STATE_IA32_PAT:           u32 = 0x2804;
pub const GUEST_STATE_IA32_EFER:          u32 = 0x2806;
pub const GUEST_STATE_IA32_PERF_GLOBAL:   u32 = 0x2808;
pub const GUEST_STATE_PDPTE0:             u32 = 0x280a;
pub const GUEST_STATE_PDPTE1:             u32 = 0x280c;
pub const GUEST_STATE_PDPTE2:             u32 = 0x280e;
pub const GUEST_STATE_PDPTE3:             u32 = 0x2810;
pub const GUEST_STATE_IA32_BNDCFG:        u32 = 0x2812;

pub const HOST_STATE_IA32_PAT:            u32 = 0x2c00;
pub const HOST_STATE_IA32_EFER:           u32 = 0x2c02;
pub const HOST_STATE_IA32_PERF_GLOBAL:    u32 = 0x2c04;

// 32-bit fields
pub const EXEC_CTRL_PINBASED:             u32 = 0x4000;
pub const EXEC_CTRL_PROCBASED_1:          u32 = 0x4002;
pub const EXEC_CTRL_EXCP_BITMAP:          u32 = 0x4004;
pub const EXEC_CTRL_PF_ERR_MASK:          u32 = 0x4006;
pub const EXEC_CTRL_PF_ERR_MATCH:         u32 = 0x4008;
pub const EXEC_CTRL_CR3_TARGET_COUNT:     u32 = 0x400a;
pub const EXIT_CTRLS:                     u32 = 0x400c;
pub const EXIT_CTRL_MSR_STORE_COUNT:      u32 = 0x400e;
pub const EXIT_CTRL_MSR_LOAD_COUNT:       u32 = 0x4010;
pub const ENTRY_CTRLS:                    u32 = 0x4012;
pub const ENTRY_CTRL_MSR_LOAD_COUNT:      u32 = 0x4014;
pub const ENTRY_CTRL_INT_INFO:            u32 = 0x4016;
pub const ENTRY_CTRL_EXCP_ERR_CODE:       u32 = 0x4018;
pub const ENTRY_CTRL_INSN_LEN:            u32 = 0x401a;
pub const EXEC_CTRL_TPR_THRESHOLD:        u32 = 0x401c;
pub const EXEC_CTRL_PROCBASED_2:          u32 = 0x401e;
pub const EXEC_CTRL_PLE_GAP:              u32 = 0x4020;
pub const EXEC_CTRL_PLE_WIN:              u32 = 0x4022;

pub const EXIT_INFO_VM_INSN_ERROR:        u32 = 0x4400;
pub const EXIT_INFO_REASON:               u32 = 0x4402;
pub const EXIT_INFO_VMEXIT_INT_INFO:      u32 = 0x4404;
pub const EXIT_INFO_VMEXIT_INT_ERR:       u32 = 0x4406;
pub const EXIT_INFO_IDT_VECT_INFO:        u32 = 0x4408;
pub const EXIT_INFO_IDT_VECT_ERR:         u32 = 0x440a;
pub const EXIT_INFO_VMEXIT_INSN_LEN:      u32 = 0x440c;
pub const EXIT_INFO_VMEXIT_INSN_INFO:     u32 = 0x440e;

pub const GUEST_STATE_ES_LIMIT:           u32 = 0x4800;
pub const GUEST_STATE_CS_LIMIT:           u32 = 0x4802;
pub const GUEST_STATE_SS_LIMIT:           u32 = 0x4804;
pub const GUEST_STATE_DS_LIMIT:           u32 = 0x4806;
pub const GUEST_STATE_FS_LIMIT:           u32 = 0x4808;
pub const GUEST_STATE_GS_LIMIT:           u32 = 0x480a;
pub const GUEST_STATE_LDTR_LIMIT:         u32 = 0x480c;
pub const GUEST_STATE_TR_LIMIT:           u32 = 0x480e;
pub const GUEST_STATE_GDTR_LIMIT:         u32 = 0x4810;
pub const GUEST_STATE_IDTR_LIMIT:         u32 = 0x4812;
pub const GUEST_STATE_ES_ACCESS_RIGHTS:   u32 = 0x4814;
pub const GUEST_STATE_CS_ACCESS_RIGHTS:   u32 = 0x4816;
pub const GUEST_STATE_SS_ACCESS_RIGHTS:   u32 = 0x4818;
pub const GUEST_STATE_DS_ACCESS_RIGHTS:   u32 = 0x481a;
pub const GUEST_STATE_FS_ACCESS_RIGHTS:   u32 = 0x481c;
pub const GUEST_STATE_GS_ACCESS_RIGHTS:   u32 = 0x481e;
pub const GUEST_STATE_LDTR_ACCESS_RIGHTS: u32 = 0x4820;
pub const GUEST_STATE_TR_ACCESS_RIGHTS:   u32 = 0x4822;
pub const GUEST_STATE_INT_STATE:          u32 = 0x4824;
pub const GUEST_STATE_ACTIVITY_STATE:     u32 = 0x4826;
pub const GUEST_STATE_SMBASE:             u32 = 0x4828;
pub const GUEST_STATE_IA32_SYSENTER_CS:   u32 = 0x482a;
pub const GUEST_STATE_PREEMPT_TIMER:      u32 = 0x482e;

pub const HOST_STATE_IA32_SYSENTER_CS:    u32 = 0x4c00;

// Natural fields
pub const EXEC_CTRL_CR0_GUEST_HOST_MASK:  u32 = 0x6000;
pub const EXEC_CTRL_CR4_GUEST_HOST_MASK:  u32 = 0x6002;
pub const EXEC_CTRL_CR0_READ_SHADOW:      u32 = 0x6004;
pub const EXEC_CTRL_CR4_READ_SHADOW:      u32 = 0x6006;
pub const EXEC_CTRL_CR3_TARGET_0:         u32 = 0x6008;
pub const EXEC_CTRL_CR3_TARGET_1:         u32 = 0x600a;
pub const EXEC_CTRL_CR3_TARGET_2:         u32 = 0x600c;
pub const EXEC_CTRL_CR3_TARGET_3:         u32 = 0x600e;

pub const EXIT_INFO_QUALIFICATION:        u32 = 0x6400;
pub const EXIT_INFO_IO_RCX:               u32 = 0x6402;
pub const EXIT_INFO_IO_RSI:               u32 = 0x6404;
pub const EXIT_INFO_IO_RDI:               u32 = 0x6406;
pub const EXIT_INFO_IO_RIP:               u32 = 0x6408;
pub const EXIT_INFO_GUEST_LINEAR_ADDR:    u32 = 0x640a;

pub const GUEST_STATE_CR0:                u32 = 0x6800;
pub const GUEST_STATE_CR3:                u32 = 0x6802;
pub const GUEST_STATE_CR4:                u32 = 0x6804;
pub const GUEST_STATE_ES_BASE:            u32 = 0x6806;
pub const GUEST_STATE_CS_BASE:            u32 = 0x6808;
pub const GUEST_STATE_SS_BASE:            u32 = 0x680a;
pub const GUEST_STATE_DS_BASE:            u32 = 0x680c;
pub const GUEST_STATE_FS_BASE:            u32 = 0x680e;
pub const GUEST_STATE_GS_BASE:            u32 = 0x6810;
pub const GUEST_STATE_TR_BASE:            u32 = 0x6814;
pub const GUEST_STATE_LDTR_BASE:          u32 = 0x6812;
pub const GUEST_STATE_GDTR_BASE:          u32 = 0x6816;
pub const GUEST_STATE_IDTR_BASE:          u32 = 0x6818;
pub const GUEST_STATE_DR7:                u32 = 0x681a;
pub const GUEST_STATE_RSP:                u32 = 0x681c;
pub const GUEST_STATE_RIP:                u32 = 0x681e;
pub const GUEST_STATE_RFLAGS:             u32 = 0x6820;
pub const GUEST_STATE_PENDING_DBG_EXCP:   u32 = 0x6822;
pub const GUEST_STATE_IA32_SYSENTER_ESP:  u32 = 0x6824;
pub const GUEST_STATE_IA32_SYSENTER_EIP:  u32 = 0x6826;

pub const HOST_STATE_CR0:                 u32 = 0x6c00;
pub const HOST_STATE_CR3:                 u32 = 0x6c02;
pub const HOST_STATE_CR4:                 u32 = 0x6c04;
pub const HOST_STATE_FS_BASE:             u32 = 0x6c06;
pub const HOST_STATE_GS_BASE:             u32 = 0x6c08;
pub const HOST_STATE_TR_BASE:             u32 = 0x6c0a;
pub const HOST_STATE_GDTR_BASE:           u32 = 0x6c0c;
pub const HOST_STATE_IDTR_BASE:           u32 = 0x6c0e;
pub const HOST_STATE_IA32_SYSENTER_ESP:   u32 = 0x6c10;
pub const HOST_STATE_IA32_SYSENTER_EIP:   u32 = 0x6c12;
pub const HOST_STATE_RSP:                 u32 = 0x6c14;
pub const HOST_STATE_RIP:                 u32 = 0x6c16;
