use core::ptr::Unique;
use spin::Mutex;

use info::data::InformationData;

pub struct InfoPointer(Unique<InformationData>);

impl InfoPointer {
    pub const fn new(addr: u64) -> InfoPointer {
        InfoPointer(unsafe { Unique::new(addr as *mut _) })
    }

    pub fn relocate(&mut self, addr: u64) {
        self.0 = unsafe { Unique::new(addr as *mut _) };
    }

    pub fn data_ref(&self) -> &InformationData {
        unsafe { self.0.as_ref() }
    }

    // XXX: Unsafe interior mutability
    //  Use UnsafeCell Instead because the compiler
    // may optimize our code as read only thinking we don't have
    // legit interior mutability. UnsafeCell tells LLVM about that.
    // https://ricardomartins.cc/2016/07/11/interior-mutability-behind-the-curtain
    pub fn as_ptr(&mut self) -> *mut InfoPointer {
        self as *const InfoPointer as *mut InfoPointer
    }

    pub fn data_mut(&mut self) -> &mut InformationData {
        unsafe { self.0.as_mut() }
    }

    pub fn data_static_mut(&mut self) -> &'static mut InformationData {
        unsafe { &mut *(*self.as_ptr()).0.as_ptr() }
    }

    // XXX: implement lock with panic() to debug location where lock is held
    pub fn lock(&self) -> &mut InfoPointer {
        log!("info.lock()\n");
        unsafe { &mut *(self as *const InfoPointer as *mut InfoPointer) }
    }
}

//pub static INFO: Mutex<InfoPointer> = Mutex::new(InfoPointer::new(0));
pub static INFO: InfoPointer = InfoPointer::new(0);

pub fn info_data() -> &'static mut InformationData {
    let info = INFO.lock();
    info.data_static_mut()
}
