// Page mapper for native and nested paging
use utils;
use vmm;
use paging::utils::*;
use paging::ptb::*;
use pool::{PageAllocator,PagePool};
use core::fmt::Debug;

#[derive(Debug,Copy,Clone)]
pub enum MapOp {
    Map,
    Unmap,
    Remap,
}

pub trait PageMapper {
    type Allocator: PageAllocator;

    type L4T: PTBMap<Entry=Self::L4E,Alloc=Self::Allocator>;
    type L3T: PTBMap<Entry=Self::L3E,Alloc=Self::Allocator>;
    type L2T: PTBMap<Entry=Self::L2E,Alloc=Self::Allocator>;
    type L1T: PTBMap<Entry=Self::L1E,Alloc=Self::Allocator>;

    type L4E: PTBEntry<Next=Self::L3T,Alloc=Self::Allocator>+Debug;
    type L3E: PTBEntry<Next=Self::L2T,Alloc=Self::Allocator>+Debug;
    type L2E: PTBEntry<Next=Self::L1T,Alloc=Self::Allocator>+Debug;
    type L1E: PTBEntry<Next=Self::L1T,Alloc=Self::Allocator>+Debug;


    // To be implemented
    fn root(&self) -> &Self::L4T;
    fn root_mut(&mut self) -> &mut Self::L4T;


    // Get requested level entry

    fn resolve_l4e<'a>(root: &'a mut Self::L4T, addr: u64)
                       -> &'a mut Self::L4E {
        root.at_mut(addr)
    }

    fn resolve_l3e<'a>(root: &'a mut Self::L4T, addr: u64,
                       conf: &PagingConfig, alloc: &mut Self::Allocator)
                       -> &'a mut Self::L3E
        where Self::L3T: 'a,
              Self::L4E: 'a {

        Self::resolve_l4e(root, addr).next(addr, conf, alloc)
    }

    fn resolve_l2e<'a>(root: &'a mut Self::L4T, addr: u64,
                       conf: &PagingConfig, alloc: &mut Self::Allocator)
                       -> &'a mut Self::L2E
        where Self::L2T: 'a, Self::L3T: 'a,
              Self::L3E: 'a, Self::L4E: 'a {

        Self::resolve_l3e(root, addr, conf, alloc).next(addr, conf, alloc)
    }

    fn resolve_l1e<'a>(root: &'a mut Self::L4T, addr: u64,
                       conf: &PagingConfig, alloc: &mut Self::Allocator)
                       -> &'a mut Self::L1E
        where Self::L1T: 'a, Self::L2T: 'a, Self::L3T: 'a,
              Self::L2E: 'a, Self::L3E: 'a, Self::L4E: 'a {

        Self::resolve_l2e(root, addr, conf, alloc).next(addr, conf, alloc)
    }

    // Generic low level operator working at Page Table Entry Level
    fn do_op_entry<T>(entry: &mut T, addr: u64,
                      conf: &PagingConfig, alloc: &mut Self::Allocator, op: MapOp)
        where T: PTBEntry<Alloc=Self::Allocator> {

        match op {
            MapOp::Map   => entry.map(  addr, conf, alloc),
            MapOp::Unmap => entry.unmap(addr, conf, alloc),
            MapOp::Remap => entry.remap(addr, conf, alloc),
        }
    }

    fn do_op(root: &mut Self::L4T, addr: u64,
             conf: &PagingConfig, alloc: &mut Self::Allocator,
             op: MapOp, lv: PagingLevel) {
        // log!("{:#?} {:#?} {:#x}\n", lv, op, addr);
        match lv {
            PagingLevel::L4 => Self::do_op_entry(
                Self::resolve_l4e(root, addr),
                addr, conf, alloc, op),

            PagingLevel::L3 => Self::do_op_entry(
                Self::resolve_l3e(root, addr, conf, alloc),
                addr, conf, alloc, op),

            PagingLevel::L2 => Self::do_op_entry(
                Self::resolve_l2e(root, addr, conf, alloc),
                addr, conf, alloc, op),

            PagingLevel::L1 => Self::do_op_entry(
                Self::resolve_l1e(root, addr, conf, alloc),
                addr, conf, alloc, op),
        }
    }


    // Recursive mapper forward/backward operators
    //
    // XXX: instead of taking root: Self::L4T, better be generic
    // Take a table and a level indicator, and on recursive call
    // give next table or something like that

    fn rec_op_bw(start: u64, end: u64, upper: u64, root: &mut Self::L4T,
                 conf: &PagingConfig, alloc: &mut Self::Allocator,
                 op: MapOp, lv: PagingLevel) -> u64 {

        let psz = pg_level_size(lv);
        let mut addr = match pg_level_next(lv) {
            None => utils::align(start, psz),

            Some(next) => {
                let start_up = utils::align_next(start, psz);
                let psh = pg_level_shift(lv);
                let diff_tbl = {
                    let s_idx = pg_abs_idx(psh, start);
                    let e_idx = pg_abs_idx(psh, end);
                    s_idx != e_idx
                };

                if diff_tbl && utils::aligned(start, psz) {
                    Self::do_op(root, start, conf, alloc, op, lv);
                    start_up
                } else {
                    Self::rec_op_bw(start, end, start_up, root,
                                    conf, alloc, op, next)
                }
            },
        };

        while addr < utils::min(utils::align(end, psz), upper) {
            Self::do_op(root, addr, conf, alloc, op, lv);
            addr += psz as u64;
        }

        addr
    }

    fn rec_op_fw(start: u64, end: u64, root: &mut Self::L4T,
                 conf: &PagingConfig, alloc: &mut Self::Allocator,
                 op: MapOp, lv: PagingLevel) {

        let psz = pg_level_size(lv);
        let mut addr = start;

        while addr < utils::align(end, psz) {
            Self::do_op(root, addr, conf, alloc, op, lv);
            addr += psz as u64;
        }

        if let Some(nxt_lv) = pg_level_next(lv) {
            if ! utils::aligned(end, psz) {
                Self::rec_op_fw(addr, end, root, conf, alloc, op, nxt_lv);
            }
        }
    }



    // Wrappers to mapper forward/backward operators

    fn forward(start: u64, end: u64, root: &mut Self::L4T,
               conf: &PagingConfig, alloc: &mut Self::Allocator,
               op: MapOp) {
        Self::rec_op_fw(start, end, root, conf, alloc, op, PagingLevel::L4);
    }

    fn backward(start: u64, end: u64, root: &mut Self::L4T,
                conf: &PagingConfig, alloc: &mut Self::Allocator,
                op: MapOp) -> u64 {
        Self::rec_op_bw(start, end, conf.map_top, root,
                        conf, alloc, op, PagingLevel::L4)
    }

    fn operate(&mut self, start: u64, end: u64, op: MapOp,
               conf: &PagingConfig, alloc: &mut Self::Allocator) {
        let fw = Self::backward(start, end, self.root_mut(), conf, alloc, op);
        Self::forward(fw, end, self.root_mut(), conf, alloc, op);
    }


    // Public API
    fn map(&mut self, start: u64, end: u64,
           conf: &PagingConfig, alloc: &mut Self::Allocator) {
        log!("map [0x{:x} - 0x{:x}]\n", start, end);
        self.operate(start, end, MapOp::Map, conf, alloc);
    }

    fn unmap(&mut self, start: u64, end: u64,
             conf: &PagingConfig, alloc: &mut Self::Allocator) {
        log!("unmap [0x{:x} - 0x{:x}]\n", start, end);
        self.operate(start, end, MapOp::Unmap, conf, alloc);
    }

    fn remap(&mut self, start: u64, end: u64,
             conf: &PagingConfig, alloc: &mut Self::Allocator) {
        log!("remap [0x{:x} - 0x{:x}]\n", start, end);
        self.operate(start, end, MapOp::Remap, conf, alloc);
    }
}
