use std::ffi::CStr;
use std::ops::{Deref, DerefMut};
use std::os::raw::c_void;
use std::pin::Pin;
use once_cell::sync::Lazy;
use smallbox::{SmallBox, smallbox};
use smallbox::space::S8;
use crate::bindings::{alloc_func, console_func, duk_api_console_init, duk_api_git_branch, duk_api_git_commit, duk_api_git_describe, duk_api_version, duk_create_heap, duk_destroy_heap, fatal_handler, free_func, realloc_func};
use crate::ctx::{DukContext};
use crate::{NoopInterop, JsInterop, JsError};

// using SmallBox with trait pointer to avoid generics in JsEngine definition
pub (crate) type InteropRef = SmallBox<dyn JsInterop, S8>;

#[derive(Debug)]
pub (crate) struct Userdata {
    pub (crate) interop: InteropRef,
}

#[derive(Debug)]
pub struct JsEngine {
    ctx: DukContext,
    inner: Pin<Box<Userdata>>,
}


/// SAFETY: JsEngine is Send and Sync since it owns Duktape heap.
/// A Duktape heap can only be accessed by one native thread at a time [(thread-safety)](https://github.com/svaarala/duktape/blob/master/doc/threading.rst#only-one-active-native-thread-at-a-time-per-duktape-heap)
/// Rust ownership system ensures that JsEngine is not shared between threads without synchronization.
unsafe impl Send for JsEngine {}
unsafe impl Sync for JsEngine {}

impl JsEngine {
    pub fn new() -> Result<Self, JsError> {
        Self::with_interop(NoopInterop)
    }

    pub fn with_interop<I: JsInterop>(interop: I) -> Result<Self, JsError> {
        let userdata = Box::pin(Userdata {
            interop: smallbox!(interop),
        });
        let udata = &(*userdata.as_ref()) as *const Userdata;

        let ctx = unsafe {
            duk_create_heap(
                Some(alloc_func),
                Some(realloc_func),
                Some(free_func),
                udata as *mut c_void,
                Some(fatal_handler))
        };

        if ctx.is_null() {
            return Err(JsError::from("Could not create duktape context".to_string()));
        }

        let e = JsEngine {
            ctx: unsafe { DukContext::from_raw(ctx) },
            inner: userdata,
        };

        unsafe {
            duk_api_console_init(ctx, Some(console_func));
        }

        Ok(e)
    }

    pub fn version() -> u32 {
        static DUK_VERSION: Lazy<u32> = Lazy::new(|| {
            unsafe { duk_api_version() }
        });
        *DUK_VERSION
    }

    pub fn version_info() -> &'static str {
        static DUK_VERSION_INFO: Lazy<String> = Lazy::new(|| {
            unsafe {
                format!(
                    "{} ({}/{})",
                    CStr::from_ptr(duk_api_git_describe()).to_str().unwrap(),
                    CStr::from_ptr(duk_api_git_branch()).to_str().unwrap(),
                    &(CStr::from_ptr(duk_api_git_commit()).to_str().unwrap())[0..9])
            }
        });
        &*DUK_VERSION_INFO
    }

    pub fn interop(&self) -> Pin<&dyn JsInterop> {
        unsafe { self.inner.as_ref().map_unchecked(|r| &*((*r).interop)) }
    }

    pub fn interop_as<I: JsInterop>(&self) -> Pin<&I> {
        unsafe { self.interop().map_unchecked(|r| r.downcast_ref::<I>().unwrap()) }
    }

    pub fn interop_mut(&mut self) -> Pin<&mut dyn JsInterop> {
        unsafe { self.inner.as_mut().map_unchecked_mut(|r| &mut *((*r).interop)) }
    }

    pub fn interop_as_mut<I: JsInterop>(&mut self) -> Pin<&mut I> {
        unsafe { self.interop_mut().map_unchecked_mut(|r| r.downcast_mut::<I>().unwrap()) }
    }

    pub fn ctx(&mut self) -> &mut DukContext {
        &mut self.ctx
    }
}

impl Drop for JsEngine {
    fn drop(&mut self) {
        if !self.ctx.ctx.is_null() {
            unsafe { duk_destroy_heap(self.ctx.ctx); }
            self.ctx.ctx = std::ptr::null_mut();
        }
    }
}

impl Deref for JsEngine {
    type Target = DukContext;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl DerefMut for JsEngine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}

#[cfg(test)]
mod tests {
    use crate::JsEngine;

    #[test]
    fn test_trait_bounds() {
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<JsEngine>();
    }

    #[test]
    fn test_version() {
        let version = JsEngine::version();
        assert_eq!(version, 20700);
    }

    #[test]
    fn test_version_info() {
        let version_info = JsEngine::version_info();
        assert_eq!(version_info, "03d4d72-dirty (HEAD/03d4d728f)");
    }

    #[test]
    fn test_move_to_other_thread() {
        let mut engine = JsEngine::new().unwrap();
        engine.push_string("Hello, World!");
        engine = std::thread::spawn(move || {
            assert_eq!(engine.get_string(-1), "Hello, World!");
            engine.push_string("Hello, Again!");
            assert!(engine.get_stack_dump().contains("ctx: top=2"));
            engine
        }).join().unwrap();

        assert_eq!(engine.get_string(-1), "Hello, Again!");
        assert_eq!(engine.get_string(-2), "Hello, World!");

        assert!(engine.get_stack_dump().contains("ctx: top=2"));
        engine.pop_n(2);
        assert!(engine.get_stack_dump().contains("ctx: top=0"));
    }
}