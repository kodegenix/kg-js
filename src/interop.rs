use std::any::TypeId;
use log::log;
use crate::{ConsoleFunc, JsEngine, JsError, Return};

pub trait JsInterop: std::any::Any + std::fmt::Debug + 'static {
    fn call(&mut self, engine: &mut JsEngine, func_name: &str) -> Result<Return, JsError>;

    unsafe fn alloc(&mut self, size: usize) -> *mut u8 {
        super::alloc::alloc(size)
    }

    unsafe fn realloc(&mut self, ptr: *mut u8, size: usize) -> *mut u8 {
        super::alloc::realloc(ptr, size)
    }

    unsafe fn free(&mut self, ptr: *mut u8) {
        super::alloc::free(ptr)
    }

    fn fatal(&mut self, msg: &str) -> ! {
        panic!("Duktape fatal error: {}", msg);
    }

    fn console(&mut self, func: ConsoleFunc, msg: &str) {
        log!(func.level(), "JS: {}", msg);
    }
}

impl dyn JsInterop {
    pub fn downcast_ref<T: JsInterop>(&self) -> Option<&T> {
        if self.type_id() == TypeId::of::<T>() {
            unsafe { Some(&*(self as *const dyn JsInterop as *const T)) }
        } else {
            None
        }
    }

    pub fn downcast_mut<T: JsInterop>(&mut self) -> Option<&mut T> {
        let t = (self as &dyn JsInterop).type_id();
        if t == TypeId::of::<T>() {
            unsafe { Some(&mut *(self as *mut dyn JsInterop as *mut T)) }
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct NoopInterop;

impl JsInterop for NoopInterop {
    fn call(&mut self, _engine: &mut JsEngine, _func_name: &str) -> Result<Return, JsError> {
        Ok(Return::Undefined)
    }
}