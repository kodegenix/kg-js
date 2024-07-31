use crate::{ConsoleFunc, JsEngine, JsError, Return};
use log::log;
use std::any::TypeId;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct Interop {
        stdout: String,
        number: f64,
    }

    impl JsInterop for Interop {
        fn call(&mut self, engine: &mut JsEngine, func_name: &str) -> Result<Return, JsError> {
            match func_name {
                "add" => {
                    let a = engine.get_number(0);
                    let b = engine.get_number(1);
                    let res = a + b;
                    engine.push_number(res);
                    Ok(Return::Top)
                }
                "sub" => {
                    let a = engine.get_number(0);
                    let b = engine.get_number(1);
                    let res = a - b;
                    engine.push_number(res);
                    Ok(Return::Top)
                }
                "put_number" => {
                    let n = engine.get_number(0);
                    self.number = n;
                    Ok(Return::Undefined)
                }
                "get_number" => {
                    engine.push_number(self.number);
                    Ok(Return::Top)
                }
                _ => unreachable!(),
            }
        }

        fn console(&mut self, _func: ConsoleFunc, msg: &str) {
            self.stdout.push_str(msg);
            self.stdout.push('\n');
        }
    }

    fn init() -> JsEngine {
        let mut e = JsEngine::with_interop(Interop::default());
        e.put_global_function("add", 2);
        e.put_global_function("sub", 2);
        e.put_global_function("put_number", 1);
        e.put_global_function("get_number", 0);
        e
    }

    #[test]
    fn call_rust_function() {
        let mut e = init();

        e.eval("var a = add(10, 11); put_number(a);").unwrap();
        assert_eq!(21f64, e.interop_as::<Interop>().number);

        e.eval("var b = sub(12, 10); put_number(b);").unwrap();
        assert_eq!(2f64, e.interop_as::<Interop>().number);

        e.eval("put_number(123.5); console.log(get_number());")
            .unwrap();
        assert_eq!("123.5\n", e.interop_as::<Interop>().stdout);
    }
}
