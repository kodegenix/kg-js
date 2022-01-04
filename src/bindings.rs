use std::mem::ManuallyDrop;
use bitflags::bitflags;
use super::*;

bitflags! {
    pub struct DukCompileFlags: u32 {
        const DUK_COMPILE_EVAL                  = (1 << 3);    /* compile eval code (instead of global code) */
        const DUK_COMPILE_FUNCTION              = (1 << 4);    /* compile function code (instead of global code) */
        const DUK_COMPILE_STRICT                = (1 << 5);    /* use strict (outer) context for global, eval, or function code */
        const DUK_COMPILE_SHEBANG               = (1 << 6);    /* allow shebang ('#! ...') comment on first line of source */
        const DUK_COMPILE_SAFE                  = (1 << 7);    /* (internal) catch compilation errors */
        const DUK_COMPILE_NORESULT              = (1 << 8);    /* (internal) omit eval result */
        const DUK_COMPILE_NOSOURCE              = (1 << 9);    /* (internal) no source string on stack */
        const DUK_COMPILE_STRLEN                = (1 << 10);   /* (internal) take strlen() of src_buffer (avoids double evaluation in macro) */
        const DUK_COMPILE_NOFILENAME            = (1 << 11);   /* (internal) no filename on stack */
        const DUK_COMPILE_FUNCEXPR              = (1 << 12);   /* (internal) source is a function expression (used for Function constructor) */
    }
}

bitflags! {
    pub struct DukDefpropFlags: u32 {
        const DUK_DEFPROP_WRITABLE              = (1 << 0);    /* set writable (effective if DUK_DEFPROP_HAVE_WRITABLE set) */
        const DUK_DEFPROP_ENUMERABLE            = (1 << 1);    /* set enumerable (effective if DUK_DEFPROP_HAVE_ENUMERABLE set) */
        const DUK_DEFPROP_CONFIGURABLE          = (1 << 2);    /* set configurable (effective if DUK_DEFPROP_HAVE_CONFIGURABLE set) */
        const DUK_DEFPROP_HAVE_WRITABLE         = (1 << 3);    /* set/clear writable */
        const DUK_DEFPROP_HAVE_ENUMERABLE       = (1 << 4);    /* set/clear enumerable */
        const DUK_DEFPROP_HAVE_CONFIGURABLE     = (1 << 5);    /* set/clear configurable */
        const DUK_DEFPROP_HAVE_VALUE            = (1 << 6);    /* set value (given on value stack) */
        const DUK_DEFPROP_HAVE_GETTER           = (1 << 7);    /* set getter (given on value stack) */
        const DUK_DEFPROP_HAVE_SETTER           = (1 << 8);    /* set setter (given on value stack) */
        const DUK_DEFPROP_FORCE                 = (1 << 9);    /* force change if possible, may still fail for e.g. virtual properties */
    }
}

bitflags! {
    pub struct DukEnumFlags: u32 {
        const DUK_ENUM_INCLUDE_NONENUMERABLE    = (1 << 0);    /* enumerate non-numerable properties in addition to enumerable */
        const DUK_ENUM_INCLUDE_HIDDEN           = (1 << 1);    /* enumerate hidden symbols too (in Duktape 1.x called internal properties) */
        const DUK_ENUM_INCLUDE_SYMBOLS          = (1 << 2);    /* enumerate symbols */
        const DUK_ENUM_EXCLUDE_STRINGS          = (1 << 3);    /* exclude strings */
        const DUK_ENUM_OWN_PROPERTIES_ONLY      = (1 << 4);    /* don't walk prototype chain, only check own properties */
        const DUK_ENUM_ARRAY_INDICES_ONLY       = (1 << 5);    /* only enumerate array indices */
        const DUK_ENUM_SORT_ARRAY_INDICES       = (1 << 6);    /* sort array indices (applied to full enumeration result, including inherited array indices) */
        const DUK_ENUM_NO_PROXY_BEHAVIOR        = (1 << 7);    /* enumerate a proxy object itself without invoking proxy behavior */
    }
}

bitflags! {
    pub struct DukBufFlags: u32 {
        const DUK_BUF_FLAG_DYNAMIC              = (1 << 0);    /* internal flag: dynamic buffer */
        const DUK_BUF_FLAG_EXTERNAL             = (1 << 1);    /* internal flag: external buffer */
        const DUK_BUF_FLAG_NOZERO               = (1 << 2);    /* internal flag: don't zero allocated buffer */
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(i32)]
#[allow(non_camel_case_types, dead_code)]
pub enum DukType {
    DUK_TYPE_NONE                     = 0,    /* no value, e.g. invalid index */
    DUK_TYPE_UNDEFINED                = 1,    /* Ecmascript undefined */
    DUK_TYPE_NULL                     = 2,    /* Ecmascript null */
    DUK_TYPE_BOOLEAN                  = 3,    /* Ecmascript boolean: 0 or 1 */
    DUK_TYPE_NUMBER                   = 4,    /* Ecmascript number: double */
    DUK_TYPE_STRING                   = 5,    /* Ecmascript string: CESU-8 / extended UTF-8 encoded */
    DUK_TYPE_OBJECT                   = 6,    /* Ecmascript object: includes objects, arrays, functions, threads */
    DUK_TYPE_BUFFER                   = 7,    /* fixed or dynamic, garbage collected byte buffer */
    DUK_TYPE_POINTER                  = 8,    /* raw void pointer */
    DUK_TYPE_LIGHTFUNC                = 9,    /* lightweight function pointer */
}

impl From<i32> for DukType {
    fn from(e: i32) -> Self {
        if e >= DukType::DUK_TYPE_NONE as i32 && e <= DukType::DUK_TYPE_LIGHTFUNC as i32 {
            unsafe { std::mem::transmute(e) }
        } else {
            panic!("incorrect DukType value: {}", e); //FIXME (jc)
        }
    }
}


#[allow(non_camel_case_types)]
pub enum duk_context {}

#[allow(non_camel_case_types)]
pub type duk_fatal_function = extern "C" fn (udata: *mut c_void, msg: *const c_char);
#[allow(non_camel_case_types)]
pub type duk_alloc_function = extern "C" fn (udata: *mut c_void, size: usize) -> *mut c_void;
#[allow(non_camel_case_types)]
pub type duk_realloc_function = extern "C" fn (udata: *mut c_void, ptr: *mut c_void, size: usize) -> *mut c_void;
#[allow(non_camel_case_types)]
pub type duk_free_function = extern "C" fn (udata: *mut c_void, ptr: *mut c_void);

#[allow(non_camel_case_types)]
pub type duk_c_function = extern "C" fn(ctx: *mut duk_context) -> i32;

#[allow(non_camel_case_types)]
pub type duk_console_function = extern "C" fn(udata: *mut c_void, fun: u32, msg: *const c_char, msg_len: usize);

#[allow(dead_code)]
extern "C" {
    pub fn duk_api_version() -> u32;
    pub fn duk_api_git_describe() -> *const c_char;
    pub fn duk_api_git_commit() -> *const c_char;
    pub fn duk_api_git_branch() -> *const c_char;
    pub fn duk_api_get_heap_udata(ctx: *mut duk_context) -> *mut c_void;
    pub fn duk_api_console_init(ctx: *mut duk_context, console_func: Option<duk_console_function>);

    pub fn duk_create_heap(alloc_func: Option<duk_alloc_function>,
                       realloc_func: Option<duk_realloc_function>,
                       free_func: Option<duk_free_function>,
                       heap_udata: *mut c_void,
                       fatal_handler: Option<duk_fatal_function>)
                       -> *mut duk_context;

    pub fn duk_destroy_heap(ctx: *mut duk_context);

    pub fn duk_eval_raw(ctx: *mut duk_context, code: *const c_char, len: usize, flags: u32) -> i32;
    pub fn duk_compile_raw(ctx: *mut duk_context, code: *const c_char, len: usize, flags: u32) -> i32;

    pub fn duk_call(ctx: *mut duk_context, nargs: i32);
    pub fn duk_call_method(ctx: *mut duk_context, nargs: i32);
    pub fn duk_call_prop(ctx: *mut duk_context, obj_index: i32, nargs: i32);
    pub fn duk_pcall(ctx: *mut duk_context, nargs: i32) -> i32;
    pub fn duk_pcall_method(ctx: *mut duk_context, nargs: i32) -> i32;
    pub fn duk_pcall_prop(ctx: *mut duk_context, obj_index: i32, nargs: i32) -> i32;

    pub fn duk_safe_to_lstring(ctx: *mut duk_context,
                           index: i32,
                           out_len: *mut usize)
                           -> *const c_char;

    pub fn duk_get_top(ctx: *mut duk_context) -> i32;
    pub fn duk_normalize_index(ctx: *mut duk_context, index: i32) -> i32;
    pub fn duk_require_normalize_index(ctx: *mut duk_context, index: i32) -> i32;

    pub fn duk_dup(ctx: *mut duk_context, index: i32);
    pub fn duk_remove(ctx: *mut duk_context, index: i32);

    pub fn duk_pop(ctx: *mut duk_context);
    pub fn duk_pop_1(ctx: *mut duk_context);
    pub fn duk_pop_2(ctx: *mut duk_context);
    pub fn duk_pop_n(ctx: *mut duk_context, n: i32);
    pub fn duk_swap(ctx: *mut duk_context, idx1: i32, idx2: i32);

    pub fn duk_push_null(ctx: *mut duk_context);
    pub fn duk_push_boolean(ctx: *mut duk_context, val: i32);
    pub fn duk_push_int(ctx: *mut duk_context, val: i32);
    pub fn duk_push_uint(ctx: *mut duk_context, val: u32);
    pub fn duk_push_number(ctx: *mut duk_context, val: f64);
    pub fn duk_push_lstring(ctx: *mut duk_context, str: *const c_char, len: usize) -> *const c_char;
    pub fn duk_push_array(ctx: *mut duk_context) -> i32;
    pub fn duk_push_object(ctx: *mut duk_context) -> i32;
    pub fn duk_push_pointer(ctx: *mut duk_context, p: *mut c_void);
    pub fn duk_push_buffer_raw(ctx: *mut duk_context, len: usize, dynamic: u32) -> *mut c_void;
    pub fn duk_push_c_function(ctx: *mut duk_context, func: Option<duk_c_function>, nargs: i32) -> i32;
    pub fn duk_push_c_lightfunc(ctx: *mut duk_context, func: Option<duk_c_function>, nargs: i32, length: i32, magic: i32);
    pub fn duk_push_current_function(ctx: *mut duk_context);
    pub fn duk_push_this(ctx: *mut duk_context);

    pub fn duk_config_buffer(ctx: *mut duk_context, index: i32, ptr: *mut c_void, len: usize);

    pub fn duk_get_type(ctx: *mut duk_context, index: i32) -> i32;
    pub fn duk_get_length(ctx: *mut duk_context, index: i32) -> usize;
    pub fn duk_samevalue(ctx: *mut duk_context, index1: i32, index2: i32) -> i32;

    pub fn duk_is_array(ctx: *mut duk_context, index: i32) -> i32;
    pub fn duk_is_object(ctx: *mut duk_context, index: i32) -> i32;
    pub fn duk_is_number(ctx: *mut duk_context, index: i32) -> i32;
    pub fn duk_is_string(ctx: *mut duk_context, index: i32) -> i32;
    pub fn duk_is_function(ctx: *mut duk_context, index: i32) -> i32;
    pub fn duk_is_thread(ctx: *mut duk_context, index: i32) -> i32;

    pub fn duk_to_object(ctx: *mut duk_context, index: i32);
    pub fn duk_to_number(ctx: *mut duk_context, index: i32) -> f64;
    pub fn duk_to_string(ctx: *mut duk_context, index: i32);

    pub fn duk_get_boolean(ctx: *mut duk_context, index: i32) -> i32;
    pub fn duk_get_number(ctx: *mut duk_context, index: i32) -> f64;
    pub fn duk_get_lstring(ctx: *mut duk_context, index: i32, len: Option<&mut usize>) -> *const c_char;
    pub fn duk_get_buffer(ctx: *mut duk_context, index: i32, len: Option<&mut usize>) -> *mut c_void;
    pub fn duk_get_pointer(ctx: *mut duk_context, index: i32) -> *mut c_void;

    pub fn duk_get_prop(ctx: *mut duk_context, obj_index: i32) -> i32;
    pub fn duk_put_prop(ctx: *mut duk_context, obj_index: i32) -> i32;
    pub fn duk_def_prop(ctx: *mut duk_context, obj_index: i32, flags: u32);

    pub fn duk_get_prop_lstring(ctx: *mut duk_context,
                            obj_index: i32,
                            key: *const c_char,
                            len: usize)
                            -> i32;
    pub fn duk_put_prop_lstring(ctx: *mut duk_context,
                            obj_index: i32,
                            key: *const c_char,
                            len: usize)
                            -> i32;
    pub fn duk_get_prop_index(ctx: *mut duk_context, obj_index: i32, index: u32) -> i32;
    pub fn duk_put_prop_index(ctx: *mut duk_context, obj_index: i32, index: u32) -> i32;

    pub fn duk_push_global_object(ctx: *mut duk_context);
    pub fn duk_get_global_lstring(ctx: *mut duk_context, key: *const c_char, len: usize) -> i32;
    pub fn duk_put_global_lstring(ctx: *mut duk_context, key: *const c_char, len: usize) -> i32;

    pub fn duk_enum(ctx: *mut duk_context, obj_index: i32, flags: u32);
    pub fn duk_next(ctx: *mut duk_context, enum_idx: i32, get_value: i32) -> i32;

    pub fn duk_fatal(ctx: *mut duk_context, err_code: i32, err_msg: *const c_char);
}


#[inline(always)]
unsafe fn interop<'a>(udata: *mut c_void) -> &'a mut InteropRef {
    &mut (*(udata as *mut Engine)).interop
}

#[inline(always)]
unsafe fn engine(udata: *mut c_void) -> ManuallyDrop<JsEngine> {
    let inner = Pin::new_unchecked(Box::from_raw(udata as *mut Engine));
    let ctx = inner.ctx;
    ManuallyDrop::new(JsEngine {
        ctx,
        inner,
    })
}

pub extern "C" fn alloc_func(udata: *mut c_void, size: usize) -> *mut c_void {
    unsafe {
        interop(udata).alloc(size) as *mut c_void
    }
}

pub extern "C" fn realloc_func(udata: *mut c_void, ptr: *mut c_void, size: usize) -> *mut c_void {
    unsafe {
        interop(udata).realloc(ptr as *mut u8, size) as *mut c_void
    }
}

pub extern "C" fn free_func(udata: *mut c_void, ptr: *mut c_void) {
    unsafe {
        interop(udata).free(ptr as *mut u8);
    }
}

pub extern "C" fn console_func(udata: *mut c_void, func: u32, msg: *const c_char, len: usize) {
    use std::str;
    use std::slice;
    unsafe {
        let msg = str::from_utf8_unchecked(slice::from_raw_parts(msg as *const u8, len));
        interop(udata).console(ConsoleFunc::from(func), msg);
    }
}

pub extern "C" fn func_dispatch(ctx: *mut duk_context) -> i32 {
    use std::str;
    use std::slice;
    unsafe {
        duk_push_current_function(ctx);
        duk_get_prop_lstring(ctx, -1, FUNC_NAME_PROP.as_ptr() as *const c_char, FUNC_NAME_PROP.len());
        let mut len: usize = 0;
        let ptr = duk_get_lstring(ctx, -1, Some(&mut len)) as *const u8;
        let name = str::from_utf8_unchecked(slice::from_raw_parts(ptr, len));
        duk_pop_2(ctx);
        let udata = duk_api_get_heap_udata(ctx);
        let mut e = engine(udata);
        let r = match interop(udata).call(&mut e, name) {
            Ok(r) => r,
            Err(_err) => Return::Error, //FIXME pass error message (requires changes in duktape)
        };
        r as i32
    }
}

pub extern "C" fn fatal_handler(udata: *mut c_void, msg: *const c_char) {
    unsafe {
        let msg = CStr::from_ptr(msg).to_string_lossy();
        interop(udata).fatal(&msg);
    }
}
