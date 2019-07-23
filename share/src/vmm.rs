use cpu;
use frame;
use paging::utils as pgutils;
use paging::ptb as pgptb;
use paging::map as pgmap;
use vmx::vmcs;
use segmentation;
use pool;

pub const MIN_STACK_SIZE: usize = 3 * pgutils::PG_4KB;

pub struct VMM {
    pub stack: u64,
    pub base:  u64,
    pub entry: u64,
    pub size:  usize,

    pub cpu:  cpu::HardwareCPU,
    pub pfr:  frame::FrameRegistry,
    pub pg:   pgptb::PagingEnv<'static, pgmap::PML4>,
    pub vmc:  &'static mut vmcs::VmmHardwareVMCS,
    pub seg:  &'static mut segmentation::VmmSegmentation,
    pub pool: pool::PagePool,
}
