use std::ffi::CStr;
use std::ops::{Deref, DerefMut};
use std::os::raw::c_void;
use std::pin::Pin;
use once_cell::sync::Lazy;
use smallbox::{SmallBox, smallbox};
use smallbox::space::S8;
use crate::bindings::{alloc_func, console_func, duk_api_console_init, duk_api_git_branch, duk_api_git_commit, duk_api_git_describe, duk_api_version, duk_context, duk_create_heap, duk_destroy_heap, fatal_handler, free_func, realloc_func};
use crate::ctx::{DukContextGuard, DukContext};
use crate::{NoopInterop, JsInterop};

// using SmallBox with trait pointer to avoid generics in JsEngine definition
pub (crate) type InteropRef = SmallBox<dyn JsInterop, S8>;

#[derive(Debug)]
pub (crate) struct Engine {
    pub (crate) ctx: *mut duk_context,
    pub (crate) interop: InteropRef,
}

#[derive(Debug)]
pub struct JsEngine {
    ctx: DukContext,
    inner: Pin<Box<Engine>>,
}


impl JsEngine {
    pub fn new() -> Self {
        Self::with_interop(NoopInterop)
    }

    pub fn with_interop<I: JsInterop>(interop: I) -> Self {
        let mut e = JsEngine {
            ctx: unsafe { DukContext::from_ptr(std::ptr::null_mut()) },
            inner: Box::pin(Engine {
                ctx: std::ptr::null_mut(),
                interop: smallbox!(interop),
            }),
        };

        let ctx = unsafe {
            duk_create_heap(
                Some(alloc_func),
                Some(realloc_func),
                Some(free_func),
                &(*e.inner.as_ref()) as *const Engine as *mut c_void,
                Some(fatal_handler))
        };

        if ctx.is_null() {
            panic!("Could not create duktape context");
        }

        unsafe {
            duk_api_console_init(ctx, Some(console_func));
        }

        unsafe {
            e.ctx.set_ptr(ctx);
            e.inner.as_mut().get_unchecked_mut().ctx = ctx;
        }
        e
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

    pub fn ctx(&mut self) -> DukContextGuard {
        DukContextGuard::new(self.ctx)
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