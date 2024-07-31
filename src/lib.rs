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
pub use error::*;

mod console;
mod ctx;
mod engine;
pub mod alloc;
mod interop;
mod error;

#[cfg(feature = "serde")]
pub mod ser;
#[cfg(feature = "serde")]
pub mod de;

const FUNC_NAME_PROP: &[u8] = b"name";

const DUK_EXEC_SUCCESS: i32 = 0;

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

