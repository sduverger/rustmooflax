use share::info::InformationData;
use share::utils::RawValue;
use share::vmx::vmcs::access::Access;

#[derive(Debug,Copy,Clone)]
pub enum CPUMode {
    real,
    v8086,
    protected,
    protected32,
    protected16,
    long,
    long64,
    long32,
    long16,
    legacy32,
    legacy16,
    paging,
    paging32,
    paging36,
    paging64,
}

#[derive(Debug,Copy,Clone)]
pub struct CPUState {
    lm:  bool,
    pe:  bool,
    pg:  bool,
    pae: bool,
    vm:  bool,
    csd: bool,
    csl: bool,
}

impl CPUState {
    pub fn is_prot(&self)     -> bool {self.pe}
    pub fn is_prot32(&self)   -> bool {self.is_prot() && !self.vm &&  self.csd}
    pub fn is_prot16(&self)   -> bool {self.is_prot() && !self.vm && !self.csd}

    pub fn is_long(&self)     -> bool {self.lm}
    pub fn is_long64(&self)   -> bool {self.is_long() &&  self.csl}
    pub fn is_long32(&self)   -> bool {self.is_long() && !self.csl && self.csd}
    pub fn is_long16(&self)   -> bool {self.is_long() && !self.csl && !self.csd}

    pub fn is_real(&self)     -> bool {!self.is_long() && !self.is_prot()}
    pub fn is_v8086(&self)    -> bool {!self.is_long() &&  self.is_prot() && self.vm}
    pub fn is_legacy32(&self) -> bool {!self.is_long() &&  self.is_prot32()}
    pub fn is_legacy16(&self) -> bool {!self.is_long() &&  self.is_prot16()}

    pub fn is_paged(&self)    -> bool {self.pg}
    pub fn is_paging32(&self) -> bool {self.is_paged() && !self.pae && !self.is_long()}
    pub fn is_paging36(&self) -> bool {self.is_paged() &&  self.pae && !self.is_long()}
    pub fn is_paging64(&self) -> bool {self.is_paged() &&  self.pae &&  self.is_long()}

    pub fn init(info: &mut InformationData) -> CPUState {
        CPUState {
            lm  : info.vm.vmcs.guest.ia32_efer.as_ref().ia32_a(),
            pe  : info.vm.vmcs.guest.cr0.as_ref().pe(),
            pg  : info.vm.vmcs.guest.cr0.as_ref().pg(),
            pae : info.vm.vmcs.guest.cr4.as_ref().pae(),
            vm  : info.vm.vmcs.guest.rflags.as_ref().vm(),
            csd : info.vm.vmcs.guest.cs.attr.as_ref().d(),
            csl : info.vm.vmcs.guest.cs.attr.as_ref().l(),
        }
    }

    pub fn mode(info: &mut InformationData, mode: CPUMode) -> bool {
        let state = CPUState::init(info);

        match mode {
            real        => state.is_real(),
            v8086       => state.is_v8086(),
            protected   => state.is_prot(),
            protected32 => state.is_prot32(),
            protected16 => state.is_prot16(),
            long        => state.is_long(),
            long64      => state.is_long64(),
            long32      => state.is_long32(),
            long16      => state.is_long16(),
            legacy32    => state.is_legacy32(),
            legacy16    => state.is_legacy16(),
            paging      => state.is_paged(),
            paging32    => state.is_paging32(),
            paging36    => state.is_paging36(),
            paging64    => state.is_paging64(),
        }
    }

    pub fn addr_size(info: &mut InformationData) -> usize {
        let state = CPUState::init(info);

        if      state.is_long64()                      { 64 }
        else if state.is_long32() || state.is_prot32() { 32 }
        else                                           { 16 }
    }
}
