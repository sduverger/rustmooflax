use x86_64::registers::control_regs::cr2 as cr2_read;
use cr::cr2_write;
use dr::{dr6_read,dr6_write};
use utils;
use utils::RawValue;

use vmx::regs::FixedReg;
use vmx::insn::{vmread, vmwrite};
use vmx::vmcs::enc::*;

bitfield!{
    #[derive(Default, Copy, Clone)]
    pub struct Encoding(u32);

    impl Debug;

    pub atype,_:0;
    pub index,_:9,1;
    pub ftype,_:11,10;
    pub fwidth,_:14,13;

    // we use reserved bits for vmcs synchro
    // but anytime we use vmx instructions
    // these bits are reset to 0

    pub read,set_read:15;    // (0) need vmread
    pub dirty,set_dirty:16;  // (1) need vmwrite
}

impl Encoding {
    pub fn as_u64(&self) -> u64 { self.0 as u64 }
}

// XXX: change RawValue so that
// we can use native field size
// and only convert to u64 when vmx_read/vmx_write
//
pub trait Access {
    type Field: utils::RawValue;

    // Require impl

    fn encoding(&self) -> &Encoding;
    fn encoding_mut(&mut self) -> &mut Encoding;

    // XXX: declare unsafe ?
    fn field(&self) -> &Self::Field;
    fn field_mut(&mut self) -> &mut Self::Field;


    // Default internal methods

    fn set_encoding(&mut self, v: u32) {
        self.encoding_mut().0 = v;
    }

    fn get_field_value(&self) -> u64 {
        self.field().as_u64()
    }

    fn set_field_value(&mut self, v: u64) {
        self.field_mut().update_u64(v);
    }

    fn read(&mut self) {
        if ! self.encoding().read() {
            self.force_read();
            self.encoding_mut().set_read(true);
        }
    }

    fn dirty(&mut self) {
        if ! self.encoding().dirty() {
            let enc = self.encoding_mut();
            if ! enc.read() {
                enc.set_read(true)
            }
            enc.set_dirty(true);
        }
    }

    fn clear(&mut self) {
        self.encoding_mut().set_read(false);
        self.encoding_mut().set_dirty(false);
    }

    fn flush(&mut self) {
        self.encoding_mut().set_read(false);

        if self.encoding().dirty() {
            self.encoding_mut().set_dirty(false);
            self.force_flush();
        }
    }

    // fn cond(&mut self, write: bool) {
    //     self.read();
    //     if write {
    //         self.encoding.set_dirty(true);
    //     }
    // }



    // Low level hardware access

    fn force_read(&mut self) {
        let val = vmread(self.encoding().as_u64());
        self.set_field_value(val);
    }

    fn force_flush(&self) {
        vmwrite(self.get_field_value(), self.encoding().as_u64());
    }



    // Public API

    fn as_ref(&mut self) -> &Self::Field {
        self.read();
        self.field()
    }

    fn as_mut(&mut self) -> &mut Self::Field {
        self.read();
        self.dirty();
        self.field_mut()
    }

}

#[repr(C, packed)]
#[derive(Default, Copy, Clone)]
pub struct Field<T> {
    field: T,
    encoding: Encoding,
}

impl<T> Access for Field<T> where T: utils::RawValue {
    type Field = T;

    fn encoding(&self) -> &Encoding { &self.encoding }
    fn encoding_mut(&mut self) -> &mut Encoding { &mut self.encoding }

    fn field(&self) -> &T { &self.field }
    fn field_mut(&mut self) -> &mut T { &mut self.field }
}

#[repr(C, packed)]
#[derive(Default, Copy, Clone)]
pub struct FixedField<T> {
    field: T,
    fixed: FixedReg<T>,
    encoding: Encoding,
}

impl<T> Access for FixedField<T> where T: utils::RawValue {
    type Field = T;

    fn encoding(&self) -> &Encoding { &self.encoding }
    fn encoding_mut(&mut self) -> &mut Encoding { &mut self.encoding }

    fn field(&self) -> &T { &self.field }
    fn field_mut(&mut self) -> &mut T { &mut self.field }

    fn get_field_value(&self) -> u64 {
        self.fixed.mask_u64(self.field.as_u64())
    }
}

impl<T> FixedField<T> where T: utils::RawValue {

    pub fn fixed(&self) -> &FixedReg<T> { &self.fixed }

    // Update local value while setting fixed bit setting register
    pub fn set_fixed(&mut self, fixed: FixedReg<T>) {
        self.fixed = fixed;
        let fixed_value = self.get_field_value();
        self.field.update_u64(fixed_value);
    }
}

#[repr(C, packed)]
#[derive(Default, Copy, Clone)]
pub struct FakeField<T> {
    field: T,
    encoding: Encoding,
}

impl<T> Access for FakeField<T> where T: utils::RawValue {
    type Field = T;

    fn encoding(&self) -> &Encoding { &self.encoding }
    fn encoding_mut(&mut self) -> &mut Encoding { &mut self.encoding }

    fn field(&self) -> &T { &self.field }
    fn field_mut(&mut self) -> &mut T { &mut self.field }

    fn force_read(&mut self) {
        let enc = self.encoding.0;
        let val: u64 = {
            if enc == GUEST_STATE_CR2 {
                cr2_read().0 as u64
            } else if enc == GUEST_STATE_DR6 {
                dr6_read()
            } else {
                panic!("vmcs read fake field unknown {}", enc);
            }
        };

        self.set_field_value(val);
    }

    fn force_flush(&self) {
        let enc = self.encoding.0;

        if enc == GUEST_STATE_CR2 {
            cr2_write(self.get_field_value())
        } else if enc == GUEST_STATE_DR6 {
            dr6_write(self.get_field_value())
        } else {
            panic!("vmcs write fake field unknown {}", enc);
        }
    }
}
