use std::ffi::CStr;
use std::ops::{Deref};
use std::os::raw::*;
use std::pin::Pin;

mod bindings;
use self::bindings::*;

pub use self::bindings::DukType;
pub use console::*;
pub use ctx::*;
pub use engine::*;
pub use interop::*;

#[cfg(feature = "serde")]
pub mod ser;

mod console;
mod ctx;
#[cfg(feature = "serde")]
pub mod de;
mod engine;
mod alloc;
mod interop;



const FUNC_NAME_PROP: &[u8] = b"name";

const DUK_EXEC_SUCCESS: i32 = 0;

//FIXME embed javascript error type
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct JsError(String);

impl std::fmt::Display for JsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error: {}", self.0)
    }
}

impl std::error::Error for JsError {}

impl From<String> for JsError {
    fn from(s: String) -> Self {
        JsError(s)
    }
}

impl Into<String> for JsError {
    fn into(self) -> String {
        self.0
    }
}

pub trait ReadJs {
    fn read_js(ctx: &mut DukContext, obj_index: i32) -> Result<Self, JsError>
    where
        Self: Sized;

    fn read_js_top(ctx: &mut DukContext) -> Result<Self, JsError>
    where
        Self: Sized,
    {
        let idx = ctx.normalize_index(-1);
        Self::read_js(ctx, idx)
    }
}

pub trait WriteJs {
    fn write_js(&self, ctx: &mut DukContext) -> Result<(), JsError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i32)]
pub enum Return {
    Undefined = 0,
    Top = 1,
    Error = -1,
    EvalError = -2,
    RangeError = -3,
    ReferenceError = -4,
    SyntaxError = -5,
    TypeError = -6,
    UriError = -7,
}


#[cfg(test)]
mod tests {
    use super::*;

    mod engine {
        use super::*;
        #[test]
        fn test_to_lstring_safety() {
            let mut engine = JsEngine::new();
            engine.push_string("test");
            let s = engine.safe_to_lstring(-1);
            assert_eq!(s, "test");
            engine.pop();
            assert_eq!(s, "test");
            drop(engine);
            assert_eq!(s, "test");
        }
    }

    mod interop {
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
}
