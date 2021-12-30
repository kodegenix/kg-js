use std::os::raw::*;
use std::ffi::CStr;
use std::alloc::Layout;
use std::pin::Pin;
use std::any::TypeId;
use log::{log, Level};
use once_cell::sync::Lazy;
use smallbox::{smallbox, SmallBox};
use smallbox::space::S8;

mod bindings;
use self::bindings::*;


const FUNC_NAME_PROP: &[u8] = b"name";


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


// using SmallBox with trait pointer to avoid generics in JsEngine definition
type InteropRef = SmallBox<dyn JsInterop, S8>;

#[derive(Debug)]
struct Engine {
    ctx: *mut duk_context,
    interop: InteropRef,
}

#[derive(Debug)]
pub struct JsEngine {
    ctx: *mut duk_context,
    inner: Pin<Box<Engine>>,
}

impl JsEngine {
    pub fn new() -> Self {
        Self::with_interop(DefaultInterop)
    }

    pub fn with_interop<I: JsInterop>(interop: I) -> Self {
        let mut e = JsEngine {
            ctx: std::ptr::null_mut(),
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

        e.ctx = ctx;
        unsafe { e.inner.as_mut().get_unchecked_mut().ctx = ctx; }
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

    #[inline]
    pub fn normalize_index(&self, index: i32) -> i32 {
        unsafe {
            duk_normalize_index(self.ctx, index)
        }
    }

    #[inline]
    pub fn get_top(&self) -> i32 {
        unsafe { duk_get_top(self.ctx) }
    }

    #[inline]
    pub fn dup(&mut self, index: i32) {
        unsafe {
            duk_dup(self.ctx, index);
        }
    }

    #[inline]
    pub fn remove(&mut self, index: i32) {
        unsafe {
            duk_remove(self.ctx, index);
        }
    }

    #[inline]
    pub fn pop(&mut self) {
        unsafe {
            duk_pop(self.ctx);
        }
    }

    #[inline]
    pub fn pop_n(&mut self, n: i32) {
        unsafe {
            duk_pop_n(self.ctx, n);
        }
    }

    #[inline]
    pub fn push_this(&mut self) {
        unsafe { duk_push_this(self.ctx); }
    }

    #[inline]
    pub fn push_global_object(&mut self) {
        unsafe { duk_push_global_object(self.ctx); }
    }

    #[inline]
    pub fn push_boolean(&mut self, value: bool) {
        unsafe { duk_push_boolean(self.ctx, value as i32) }
    }

    #[inline]
    pub fn push_null(&mut self) {
        unsafe { duk_push_null(self.ctx) }
    }

    #[inline]
    pub fn push_i32(&mut self, value: i32) {
        unsafe { duk_push_int(self.ctx, value) }
    }

    #[inline]
    pub fn push_u32(&mut self, value: u32) {
        unsafe { duk_push_uint(self.ctx, value) }
    }

    #[inline]
    pub fn push_number(&mut self, value: f64) {
        unsafe { duk_push_number(self.ctx, value) }
    }

    #[inline]
    pub fn push_string(&mut self, value: &str) {
        unsafe {
            duk_push_lstring(self.ctx, value.as_ptr() as *const c_char, value.len());
        }
    }

    #[inline]
    pub fn push_object(&mut self) -> i32 {
        unsafe { duk_push_object(self.ctx) }
    }

    #[inline]
    pub fn push_ext_buffer(&mut self, data: &[u8]) {
        unsafe {
            duk_push_buffer_raw(self.ctx, 0, (DukBufFlags::DUK_BUF_FLAG_DYNAMIC | DukBufFlags::DUK_BUF_FLAG_EXTERNAL).bits());
            duk_config_buffer(self.ctx, -1, data.as_ptr() as *mut c_void, data.len());
        }
    }

    #[inline]
    pub fn push_array(&mut self) -> i32 {
        unsafe { duk_push_array(self.ctx) }
    }

    pub fn push_function(&mut self, func_name: &str, nargs: i32) {
        unsafe {
            duk_push_c_function(self.ctx, Some(func_dispatch), nargs);
            duk_push_lstring(self.ctx, FUNC_NAME_PROP.as_ptr() as *const c_char, FUNC_NAME_PROP.len());
            duk_push_lstring(self.ctx, func_name.as_ptr() as *const c_char, func_name.len());
            duk_def_prop(self.ctx, -3, (DukDefpropFlags::DUK_DEFPROP_ENUMERABLE | DukDefpropFlags::DUK_DEFPROP_HAVE_VALUE).bits())
        }
    }

    pub fn put_prop_function(&mut self, obj_index: i32, func_name: &str, nargs: i32) {
        let obj_index = self.normalize_index(obj_index);
        self.push_function(func_name, nargs);
        unsafe {
            duk_put_prop_lstring(self.ctx, obj_index, func_name.as_ptr() as *const c_char, func_name.len());
        }
    }

    pub fn put_global_function(&mut self, func_name: &str, nargs: i32) {
        self.push_function(func_name, nargs);
        self.put_global_string(func_name);
    }

    #[inline]
    fn get_type(&self, index: i32) -> DukType {
        DukType::from(unsafe { duk_get_type(self.ctx, index) })
    }

    #[inline]
    pub fn is_string(&self, index: i32) -> bool {
        unsafe { duk_is_string(self.ctx, index) == 1 }
    }

    #[inline]
    pub fn is_number(&self, index: i32) -> bool {
        unsafe { duk_is_number(self.ctx, index) == 1 }
    }

    #[inline]
    pub fn is_object(&self, index: i32) -> bool {
        unsafe { duk_is_object(self.ctx, index) == 1 }
    }

    #[inline]
    pub fn is_array(&self, index: i32) -> bool {
        unsafe { duk_is_array(self.ctx, index) == 1 }
    }

    #[inline]
    pub fn is_pure_object(&self, index: i32) -> bool {
        unsafe {
            duk_is_object(self.ctx, index) == 1
            && duk_is_array(self.ctx, index) == 0
            && duk_is_function(self.ctx, index) == 0
            && duk_is_thread(self.ctx, index) == 0
        }
    }

    #[inline]
    pub fn get_string(&mut self, index: i32) -> &str {
        use std::str;
        use std::slice;
        unsafe {
            let mut len: usize = 0;
            let ptr = duk_get_lstring(self.ctx, index, Some(&mut len)) as *const u8;
            str::from_utf8_unchecked(slice::from_raw_parts(ptr, len))
        }
    }

    #[inline]
    pub fn get_buffer(&mut self, index: i32) -> &[u8] {
        use std::slice;
        unsafe {
            let mut len: usize = 0;
            let ptr = duk_get_buffer(self.ctx, index, Some(&mut len)) as *const u8;
            slice::from_raw_parts(ptr, len)
        }
    }

    #[inline]
    pub fn get_number(&mut self, index: i32) -> f64 {
        unsafe { duk_get_number(self.ctx, index) }
    }

    #[inline]
    pub fn get_boolean(&mut self, index: i32) -> bool {
        unsafe { duk_get_boolean(self.ctx, index) != 0 }
    }

    #[inline]
    pub fn get_prop(&mut self, obj_index: i32) -> bool {
        unsafe { duk_get_prop(self.ctx, obj_index) == 1 }
    }

    #[inline]
    pub fn put_prop(&mut self, obj_index: i32) {
        unsafe { duk_put_prop(self.ctx, obj_index); }
    }

    #[inline]
    pub fn get_prop_string(&mut self, obj_index: i32, key: &str) -> bool {
        unsafe {
            duk_get_prop_lstring(self.ctx, obj_index, key.as_ptr() as *const c_char, key.len()) == 1
        }
    }

    #[inline]
    pub fn put_prop_string(&mut self, obj_index: i32, key: &str) {
        unsafe {
            duk_put_prop_lstring(self.ctx,
                                 obj_index,
                                 key.as_ptr() as *const c_char,
                                 key.len());
        }
    }

    #[inline]
    pub fn get_prop_index(&mut self, obj_index: i32, index: u32) -> bool {
        unsafe { duk_get_prop_index(self.ctx, obj_index, index) == 1 }
    }

    #[inline]
    pub fn put_prop_index(&mut self, obj_index: i32, index: u32) {
        unsafe {
            duk_put_prop_index(self.ctx, obj_index, index);
        }
    }

    #[inline]
    pub fn get_global_string(&mut self, key: &str) -> bool {
        unsafe {
            duk_get_global_lstring(self.ctx, key.as_ptr() as *const c_char, key.len()) == 1
        }
    }

    #[inline]
    pub fn put_global_string(&mut self, key: &str) {
        unsafe {
            duk_put_global_lstring(self.ctx, key.as_ptr() as *const c_char, key.len());
        }
    }

    #[inline]
    fn get_length(&mut self, obj_index: i32) -> usize {
        unsafe {
            duk_get_length(self.ctx, obj_index)
        }
    }

    #[inline]
    fn enum_indices(&mut self, obj_index: i32) {
        unsafe {
            duk_enum(self.ctx, obj_index, DukEnumFlags::DUK_ENUM_ARRAY_INDICES_ONLY.bits());
        }
    }

    #[inline]
    fn enum_keys(&mut self, obj_index: i32) {
        unsafe {
            duk_enum(self.ctx, obj_index, DukEnumFlags::DUK_ENUM_OWN_PROPERTIES_ONLY.bits());
        }
    }

    #[inline]
    fn next(&mut self, obj_index: i32) -> bool {
        unsafe {
            duk_next(self.ctx, obj_index, 1) == 1
        }
    }

    #[inline]
    pub fn call_prop(&mut self, obj_index: i32, nargs: usize) {
        unsafe {
            duk_call_prop(self.ctx, obj_index, nargs as i32);
        }
    }

    #[inline]
    pub fn eval(&mut self, code: &str) -> Result<(), JsError> {
        unsafe {
            if duk_eval_raw(self.ctx,
                            code.as_ptr() as *const c_char,
                            code.len(),
                            0 | (DukCompileFlags::DUK_COMPILE_SAFE | DukCompileFlags::DUK_COMPILE_NOSOURCE | DukCompileFlags::DUK_COMPILE_NOFILENAME).bits()) != 0 {
                let mut len: usize = 0;
                let msg = duk_safe_to_lstring(self.ctx, -1, &mut len);
                let s = String::from(std::str::from_utf8_unchecked(std::slice::from_raw_parts(msg as *const u8, len)));
                duk_pop(self.ctx);
                Err(JsError(s))
            } else {
                Ok(())
            }
        }
    }

    #[inline]
    pub fn eval_file(&mut self, filename: &str, code: &str) -> Result<(), JsError> {
        unsafe {
            duk_push_lstring(self.ctx, filename.as_ptr() as *const c_char, filename.len());
            if duk_eval_raw(self.ctx,
                            code.as_ptr() as *const c_char,
                            code.len(),
                            1 | (DukCompileFlags::DUK_COMPILE_SAFE | DukCompileFlags::DUK_COMPILE_NOSOURCE).bits()) != 0 {
                let mut len: usize = 0;
                let msg = duk_safe_to_lstring(self.ctx, -1, &mut len);
                let s = String::from(std::str::from_utf8_unchecked(std::slice::from_raw_parts(msg as *const u8, len)));
                duk_pop(self.ctx);
                Err(JsError(s))
            } else {
                Ok(())
            }
        }
    }

    #[inline]
    pub fn compile(&mut self, code: &str) -> Result<(), JsError> {
        unsafe {
            if duk_compile_raw(self.ctx,
                               code.as_ptr() as *const c_char,
                               code.len(),
                               0 | (DukCompileFlags::DUK_COMPILE_NORESULT | DukCompileFlags::DUK_COMPILE_NOFILENAME).bits()) != 0 {
                let mut len: usize = 0;
                let msg = duk_safe_to_lstring(self.ctx, -1, &mut len);
                let s = String::from(std::str::from_utf8_unchecked(std::slice::from_raw_parts(msg as *const u8, len)));
                duk_pop(self.ctx);
                Err(JsError(s))
            } else {
                Ok(())
            }
        }
    }

    #[inline]
    pub fn compile_file(&mut self, filename: &str, code: &str) -> Result<(), JsError> {
        unsafe {
            duk_push_lstring(self.ctx, filename.as_ptr() as *const c_char, filename.len());
            if duk_compile_raw(self.ctx,
                               code.as_ptr() as *const c_char,
                               code.len(),
                               1 | DukCompileFlags::DUK_COMPILE_NORESULT.bits()) != 0 {
                let mut len: usize = 0;
                let msg = duk_safe_to_lstring(self.ctx, -1, &mut len);
                let s = String::from(std::str::from_utf8_unchecked(std::slice::from_raw_parts(msg as *const u8, len)));
                duk_pop(self.ctx);
                Err(JsError(s))
            } else {
                Ok(())
            }
        }
    }

    #[inline]
    pub fn write<O: WriteJs>(&mut self, obj: &O) -> Result<(), JsError> {
        obj.write_js(self)
    }

    #[inline]
    pub fn read<O: ReadJs>(&mut self, obj: &mut O, obj_index: i32) -> Result<(), JsError> {
        let obj_index = self.normalize_index(obj_index);
        obj.read_js(self, obj_index)
    }

    #[inline]
    pub fn read_top<O: ReadJs>(&mut self, obj: &mut O) -> Result<(), JsError> {
        self.read(obj, -1)
    }
}

impl Drop for JsEngine {
    fn drop(&mut self) {
        if !self.ctx.is_null() {
            unsafe { duk_destroy_heap(self.ctx); }
            self.ctx = std::ptr::null_mut();
        }
    }
}


pub trait ReadJs {
    fn read_js(&mut self, engine: &mut JsEngine, obj_index: i32) -> Result<(), JsError>;

    fn read_js_top(&mut self, engine: &mut JsEngine) -> Result<(), JsError> {
        let idx = engine.normalize_index(-1);
        self.read_js(engine, idx)
    }
}

pub trait WriteJs {
    fn write_js(&self, engine: &mut JsEngine) -> Result<(), JsError>;
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


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum ConsoleFunc {
    Assert = 1,
    Log = 2,
    Debug = 3,
    Trace = 4,
    Info = 5,
    Warn = 6,
    Error = 7,
    Exception = 8,
    Dir = 9,
}

impl ConsoleFunc {
    pub fn level(&self) -> Level {
        match *self {
            Self::Assert => Level::Error,
            Self::Log => Level::Debug,
            Self::Debug => Level::Debug,
            Self::Trace => Level::Trace,
            Self::Info => Level::Info,
            Self::Warn => Level::Warn,
            Self::Error => Level::Error,
            Self::Exception => Level::Error,
            Self::Dir => Level::Debug,
        }
    }
}

impl From<u32> for ConsoleFunc {
    fn from(value: u32) -> Self {
        match value {
            1 => Self::Assert,
            2 => Self::Log,
            3 => Self::Debug,
            4 => Self::Trace,
            5 => Self::Info,
            6 => Self::Warn,
            7 => Self::Error,
            8 => Self::Exception,
            9 => Self::Dir,
            _ => Self::Log,
        }
    }
}

pub trait JsInterop: std::any::Any + std::fmt::Debug + 'static {
    fn call(&mut self, engine: &mut JsEngine, func_name: &str) -> Result<Return, JsError>;

    unsafe fn alloc(&mut self, size: usize) -> *mut u8 {
        let layout = Layout::from_size_align_unchecked(size, std::mem::size_of::<usize>());
        std::alloc::alloc(layout)
    }

    unsafe fn realloc(&mut self, ptr: *mut u8, size: usize) -> *mut u8 {
        let layout = Layout::from_size_align_unchecked(std::mem::size_of::<usize>(), std::mem::size_of::<usize>());
        std::alloc::realloc(ptr, layout, size)
    }

    unsafe fn free(&mut self, ptr: *mut u8) {
        let layout = Layout::from_size_align_unchecked(std::mem::size_of::<usize>(), std::mem::size_of::<usize>());
        std::alloc::dealloc(ptr, layout);
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
struct DefaultInterop;

impl JsInterop for DefaultInterop {
    fn call(&mut self, _engine: &mut JsEngine, _func_name: &str) -> Result<Return, JsError> {
        Ok(Return::Undefined)
    }
}


#[cfg(feature = "serde")]
mod ser;

#[cfg(feature = "serde")]
mod de;


#[cfg(test)]
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
                _ => unreachable!()
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

        e.eval("put_number(123.5); console.log(get_number());").unwrap();
        assert_eq!("123.5\n", e.interop_as::<Interop>().stdout);
    }
}
