// System memory discovery and initialization

use core::mem;
use rlibc::memset;
use multiboot::{Multiboot, MemoryType};

use share::utils;
use share::vmm;
use share::paging::utils as pgutils;
use share::smem::{SecretArea, HardwareMemory};
use share::frame::{FrameRegistry, FrameDescriptor};
use share::smap::{SystemMap, SystemMapEntry};
use share::cpu::HardwareCPU;
use share::gpr::GPR64Context;
use share::segmentation::VmmSegmentation;
use share::vmx::vmcs::{VmmHardwareVMCS, VmHardwareVMCS};
use elf64;
use share::info;

fn inspect(mbi: &Multiboot) -> (u64, u64, usize) {
    let mmaps = match mbi.memory_regions() {
        None => panic!("no multiboot memory maps !"),
        Some(m) => m,
    };

    let mut cnt = 0;
    let mut ram_end  = 0;
    let mut area_end : Option<u64> = None;

    for m in mmaps {
        cnt += 1;

        log!("mem base {:x} length {:x} type {:?}\n"
             ,m.base_address()
             ,m.length()
             ,m.memory_type());

        if m.memory_type() != MemoryType::RAM {
            continue
        }

        if m.base_address() + m.length() > ram_end {
            ram_end = m.base_address() + m.length();
        }

        // smap entry for high mem above 1MB and below 4GB
        if m.base_address() == 1<<20 {
            area_end = Some(m.base_address() + m.length());
        }
    }

    match area_end {
        Some(end) => (end, ram_end, cnt),
        None => panic!("can't define VMM area end !")
    }
}

fn mbi_vmm_elf(mbi: &Multiboot) -> u64 {
    let modules = match mbi.modules() {
        None => panic!("no multiboot modules !"),
        Some(m) => m,
    };

    let mut fall_back = 0;
    let mut cnt = 0;
    for m in modules {
        cnt += 1;
        if cnt == 2 {
            fall_back = m.start;
        }

        let name = m.string.unwrap_or("?");

        log!("multiboot module: {} 0x{:x} 0x{:x}\n"
             ,name, m.start, m.end);

        if name.contains("vmm.bin") {
            return m.start;
        }
    }

    if fall_back != 0 {
        log!("GRUB2 ? fall back to second module !");
        return fall_back;
    }

    panic!("no vmm module found");
}

pub fn init(mbi_addr: u64) {
    // retrieve Multiboot v1 information
    let mbi = unsafe {
        match Multiboot::new(mbi_addr, utils::addr_to_option_slice) {
            None => panic!("No multiboot info found !"),
            Some(mb) => mb,
        }};

    // get vmm elf module address
    let elf_addr = mbi_vmm_elf(&mbi);

    // get physical memory layout
    let (area_end, ram_end, smap_cnt) = inspect(&mbi);
    let pfn = pgutils::pfn(ram_end);

    // compute some sizes
    let info_sz = mem::size_of::<info::InformationData>();
    let pfr_sz = mem::size_of::<FrameDescriptor>() * pfn;
    let smap_sz = mem::size_of::<SystemMapEntry>() * smap_cnt;
    let pool_sz = 200 * pgutils::PG_4KB;
    let elf_sz = elf64::module_size(elf_addr);

    let mut need_aligned =
        (vmm::MIN_STACK_SIZE
         + pool_sz
         + 2*pgutils::PML4_SZ
         + mem::size_of::<VmmHardwareVMCS>()
         + mem::size_of::<VmHardwareVMCS>()
         + mem::size_of::<VmmSegmentation>()
        ) as u64;

    // Take care of ELF alignment
    let elf_aln = elf64::module_align(elf_addr);
    if ! utils::aligned(need_aligned, elf_aln) {
        need_aligned = utils::align_next(need_aligned, elf_aln);
    }

    let mut need_uinfo = need_aligned + (elf_sz + pfr_sz + smap_sz) as u64;

    // Align Info on 8 bytes
    if ! utils::aligned(need_uinfo, mem::size_of::<u64>()) {
        need_uinfo = utils::align_next(need_uinfo, mem::size_of::<u64>());
    }

    let need = need_uinfo + (info_sz as u64);

    if need > area_end {
        panic!("not enough mem, area end {} need {}", area_end, need);
    }

    // cleanup VMM area
    let area_start = utils::align(area_end - need, pgutils::PG_4KB);
    let area_size  = (area_end - area_start) as usize;

    let secret = SecretArea {
        start: area_start,
        end:   area_end,
        size:  area_size,
    };

    unsafe { memset(secret.start as *mut u8, 0, secret.size); }


    // Step 1 - we can now relocate info pointer
    let info_addr = secret.start + need_uinfo;
    let info = {
        let info_ptr = info::INFO.lock();
        info_ptr.relocate(info_addr);
        info::info_data()
    };

    info.hwmm.setup(&secret, ram_end);
    info.hwmm.show();

    // Step 2 - allocate aligned VMM objects, and refer to them inside InfoData
    let mut addr = secret.start + (vmm::MIN_STACK_SIZE as u64);
    info.vmm.stack = addr;

    let gpr_addr = info.vmm.stack - (mem::size_of::<GPR64Context>() as u64);
    info.vmm.cpu.setup();
    info.vm.cpu.setup(&info.vmm.cpu, gpr_addr);

    info.vmm.pool.start = addr;
    info.vmm.pool.size  = pool_sz;
    info.vmm.pool.used  = 0;
    addr += info.vmm.pool.size as u64;

    info.vmm.pg.root = unsafe { &mut *(addr as *mut _) };
    addr += pgutils::PML4_SZ as u64;

    info.vm.pg.root = unsafe { &mut *(addr as *mut _) };
    addr += pgutils::PML4_SZ as u64;

    info.vmm.vmc = unsafe { &mut *(addr as *mut _) };
    addr += mem::size_of::<VmmHardwareVMCS>() as u64;

    info.vm.vmc = unsafe { &mut *(addr as *mut _) };
    addr += mem::size_of::<VmHardwareVMCS>() as u64;

    info.vmm.seg = unsafe { &mut *(addr as *mut _) };
    addr += mem::size_of::<VmmSegmentation>() as u64;


    // Step 3 - vmm ELF rebase at phdr aligned location
    info.vmm.base  = secret.start + need_aligned;
    info.vmm.size  = elf_sz;
    info.vmm.entry = info.vmm.base + elf64::module_entry(elf_addr);
    elf64::module_load(elf_addr, info.vmm.base);


    // Step 4 - set VMM info pointer
    {
        let ptr = info.vmm.base as *mut u64;
        unsafe { *ptr = info_addr; }
    } log!("VMM Info pointer = {:#x}\n", info_addr);

    // XXX: erase elf module from memory ?
    addr = info.vmm.base + elf_sz as u64;

    log!("VMM ELF: base {:#x} entry {:#x}\n",
         info.vmm.base, info.vmm.entry);


    // Step 5 - allocate unaligned VMM objects right after InfoData
    info.vmm.pfr = FrameRegistry::init(addr, &info.hwmm);
    addr += pfr_sz as u64;

    info.vm.smap = SystemMap::init(addr, smap_cnt, secret.start, &mbi);
    addr += smap_sz as u64;
}
