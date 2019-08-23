#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;

use std::ffi::CStr;
use std::os::raw::*;
use std::ptr;
use std::mem::{transmute, size_of};

use rlibc::memcpy;

use kg_tree::{NodeRef, Node, Value, Properties, Elements};


bitflags! {
    struct DukCompileFlags: u32 {
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
    struct DukDefpropFlags: u32 {
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
    struct DukEnumFlags: u32 {
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(i32)]
#[allow(non_camel_case_types, dead_code)]
enum DukType {
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
            unsafe {
                transmute(e)
            }
        } else {
            panic!(format!("Incorrect DukType value: {}", e)); //FIXME (jc)
        }
    }
}


#[allow(non_camel_case_types)]
enum duk_context {}

#[allow(non_camel_case_types)]
type duk_fatal_function = extern "C" fn(udata: *const c_void, msg: *const c_char);

#[allow(non_camel_case_types)]
type duk_c_function = extern "C" fn(ctx: *mut duk_context) -> i32;

#[allow(dead_code)]
extern "C" {
    fn duk_version() -> u32;
    fn duk_git_describe() -> *const c_char;
    fn duk_git_commit() -> *const c_char;
    fn duk_git_branch() -> *const c_char;

    fn duk_create_context(heap_udata: *const c_void,
                          fatal_handler: Option<duk_fatal_function>)
                          -> *mut duk_context;

    fn duk_create_heap(alloc_func: *const c_void,
                       realloc_func: *const c_void,
                       free_func: *const c_void,
                       heap_udata: *const c_void,
                       fatal_handler: Option<duk_fatal_function>)
                       -> *mut duk_context;

    fn duk_destroy_heap(ctx: *mut duk_context);

    fn duk_eval_raw(ctx: *mut duk_context, code: *const c_char, len: usize, flags: u32) -> i32;
    fn duk_compile_raw(ctx: *mut duk_context, code: *const c_char, len: usize, flags: u32) -> i32;

    fn duk_call(ctx: *mut duk_context, nargs: i32);
    fn duk_call_method(ctx: *mut duk_context, nargs: i32);
    fn duk_call_prop(ctx: *mut duk_context, obj_index: i32, nargs: i32);
    fn duk_pcall(ctx: *mut duk_context, nargs: i32) -> i32;
    fn duk_pcall_method(ctx: *mut duk_context, nargs: i32) -> i32;
    fn duk_pcall_prop(ctx: *mut duk_context, obj_index: i32, nargs: i32) -> i32;

    fn duk_safe_to_lstring(ctx: *mut duk_context,
                           index: i32,
                           out_len: *mut usize)
                           -> *const c_char;

    fn duk_get_top(ctx: *mut duk_context) -> i32;
    fn duk_normalize_index(ctx: *mut duk_context, index: i32) -> i32;
    fn duk_require_normalize_index(ctx: *mut duk_context, index: i32) -> i32;

    fn duk_dup(ctx: *mut duk_context, index: i32);
    fn duk_remove(ctx: *mut duk_context, index: i32);

    fn duk_pop(ctx: *mut duk_context);
    fn duk_pop_1(ctx: *mut duk_context);
    fn duk_pop_2(ctx: *mut duk_context);
    fn duk_pop_n(ctx: *mut duk_context, n: i32);

    fn duk_push_null(ctx: *mut duk_context);
    fn duk_push_boolean(ctx: *mut duk_context, val: i32);
    fn duk_push_number(ctx: *mut duk_context, val: f64);
    fn duk_push_lstring(ctx: *mut duk_context, str: *const c_char, len: usize) -> *const c_char;
    fn duk_push_array(ctx: *mut duk_context) -> i32;
    fn duk_push_object(ctx: *mut duk_context) -> i32;
    fn duk_push_pointer(ctx: *mut duk_context, p: *mut c_void);
    fn duk_push_buffer_raw(ctx: *mut duk_context, len: usize, dynamic: i32) -> *mut c_void;

    fn duk_push_c_function(ctx: *mut duk_context, func: Option<duk_c_function>, nargs: i32) -> i32;
    fn duk_push_current_function(ctx: *mut duk_context);
    fn duk_push_this(ctx: *mut duk_context);

    fn duk_get_type(ctx: *mut duk_context, index: i32) -> i32;
    fn duk_get_length(ctx: *mut duk_context, index: i32) -> usize;
    fn duk_samevalue(ctx: *mut duk_context, index1: i32, index2: i32) -> i32;

    fn duk_is_array(ctx: *mut duk_context, index: i32) -> i32;
    fn duk_is_object(ctx: *mut duk_context, index: i32) -> i32;
    fn duk_is_number(ctx: *mut duk_context, index: i32) -> i32;
    fn duk_is_string(ctx: *mut duk_context, index: i32) -> i32;
    fn duk_is_function(ctx: *mut duk_context, index: i32) -> i32;
    fn duk_is_thread(ctx: *mut duk_context, index: i32) -> i32;

    fn duk_to_object(ctx: *mut duk_context, index: i32);
    fn duk_to_number(ctx: *mut duk_context, index: i32) -> f64;
    fn duk_to_string(ctx: *mut duk_context, index: i32);

    fn duk_get_boolean(ctx: *mut duk_context, index: i32) -> i32;
    fn duk_get_number(ctx: *mut duk_context, index: i32) -> f64;
    fn duk_get_lstring(ctx: *mut duk_context, index: i32, len: Option<&mut usize>) -> *const c_char;
    fn duk_get_buffer(ctx: *mut duk_context, index: i32, len: Option<&mut usize>) -> *mut c_void;
    fn duk_get_pointer(ctx: *mut duk_context, index: i32) -> *mut c_void;

    fn duk_get_prop(ctx: *mut duk_context, obj_index: i32) -> i32;
    fn duk_put_prop(ctx: *mut duk_context, obj_index: i32) -> i32;
    fn duk_def_prop(ctx: *mut duk_context, obj_index: i32, flags: u32);

    fn duk_get_prop_lstring(ctx: *mut duk_context,
                            obj_index: i32,
                            key: *const c_char,
                            len: usize)
                            -> i32;
    fn duk_put_prop_lstring(ctx: *mut duk_context,
                            obj_index: i32,
                            key: *const c_char,
                            len: usize)
                            -> i32;
    fn duk_get_prop_index(ctx: *mut duk_context, obj_index: i32, index: u32) -> i32;
    fn duk_put_prop_index(ctx: *mut duk_context, obj_index: i32, index: u32) -> i32;

    fn duk_push_global_object(ctx: *mut duk_context);
    fn duk_get_global_lstring(ctx: *mut duk_context, key: *const c_char, len: usize) -> i32;
    fn duk_put_global_lstring(ctx: *mut duk_context, key: *const c_char, len: usize) -> i32;

    fn duk_enum(ctx: *mut duk_context, obj_index: i32, flags: u32);
    fn duk_next(ctx: *mut duk_context, enum_idx: i32, get_value: i32) -> i32;

    fn duk_fatal(ctx: *mut duk_context, err_code: i32, err_msg: *const c_char);

    fn duk_console_init(ctx: *mut duk_context, flags: u32);
}


extern "C" fn fatal_handler(udata: *const c_void, msg: *const c_char) {
    unsafe {
        let msg = CStr::from_ptr(msg).to_string_lossy();
        let s = format!("Duktape fatal error (udata {:p}): {}", udata, msg);
        error!("Duktape fatal error (udata {:p}): {}", udata, msg); //FIXME (jc)
        panic!(s); //FIXME (jc)
    }
}

const NAME_PROP: &str = "name";
const TARGET_PROP: &str = "target";

extern "C" fn func_dispatch(ctx: *mut duk_context) -> i32 {
    use std::str;
    use std::slice;
    unsafe {
        duk_push_current_function(ctx);
        duk_get_prop_lstring(ctx, -1, NAME_PROP.as_ptr() as *const c_char, NAME_PROP.len());
        let mut len: usize = 0;
        let ptr = duk_get_lstring(ctx, -1, Some(&mut len)) as *const u8;
        let name = str::from_utf8_unchecked(slice::from_raw_parts(ptr, len));
        duk_pop(ctx);
        duk_get_prop_lstring(ctx, -1, TARGET_PROP.as_ptr() as *const c_char, TARGET_PROP.len());
        let target: &mut dyn CallJs = *(duk_get_buffer(ctx, -1, None) as *mut &mut dyn CallJs);
        duk_pop_2(ctx);
        target.call(&mut Engine::from_ptr(ctx), name)
    }
}


#[derive(Debug)]
pub struct Engine {
    ctx: *mut duk_context,
    owner: bool,
}

impl Engine {
    fn from_ptr(ctx: *mut duk_context) -> Engine {
        Engine {
            ctx: ctx,
            owner: false,
        }
    }

    pub fn new() -> Engine {
        let ctx = unsafe {
            duk_create_context(ptr::null(), Some(fatal_handler))
        };

        if ctx.is_null() {
            panic!("Could not create duktape context"); //FIXME (jc)
        }

        debug!("Created duktape context: {:p}", ctx); //FIXME (jc)

        Engine {
            ctx: ctx,
            owner: true,
        }
    }

    pub fn version() -> u32 {
        lazy_static!{
            static ref DUK_VERSION: u32 = {
                unsafe {
                    duk_version()
                }
            };
        }
        *DUK_VERSION
    }

    pub fn version_info() -> &'static str {
        lazy_static!{
            static ref DUK_VERSION_INFO: String = {
                unsafe {
                    format!(
                        "{} ({}/{})",
                        CStr::from_ptr(duk_git_describe()).to_str().unwrap(),
                        CStr::from_ptr(duk_git_branch()).to_str().unwrap(),
                        &(CStr::from_ptr(duk_git_commit()).to_str().unwrap())[0..9])
                }
            };
        }
        &DUK_VERSION_INFO
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
    pub fn push_object(&mut self) -> i32 {
        unsafe { duk_push_object(self.ctx) }
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

    pub fn push_function(&mut self, target: &mut dyn CallJs, func_name: &str, nargs: i32) {
        unsafe {
            duk_push_c_function(self.ctx, Some(func_dispatch), nargs);
            duk_push_lstring(self.ctx, NAME_PROP.as_ptr() as *const c_char, NAME_PROP.len());
            duk_push_lstring(self.ctx, func_name.as_ptr() as *const c_char, func_name.len());
            duk_def_prop(self.ctx, -3, (DukDefpropFlags::DUK_DEFPROP_HAVE_VALUE | DukDefpropFlags::DUK_DEFPROP_FORCE).bits());
            duk_push_lstring(self.ctx, TARGET_PROP.as_ptr() as *const c_char, TARGET_PROP.len());
            let n = size_of::<&mut dyn CallJs>();
            let p = duk_push_buffer_raw(self.ctx, n, 0);
            memcpy(transmute(p), transmute(&target), n);
            duk_def_prop(self.ctx, -3, (DukDefpropFlags::DUK_DEFPROP_HAVE_VALUE | DukDefpropFlags::DUK_DEFPROP_FORCE).bits());
        }
    }

    pub fn put_prop_function(&mut self, obj_index: i32, target: &mut dyn CallJs, func_name: &str, nargs: i32) {
        unsafe {
            let obj_index = self.normalize_index(obj_index);
            duk_push_c_function(self.ctx, Some(func_dispatch), nargs);
            duk_push_lstring(self.ctx, NAME_PROP.as_ptr() as *const c_char, NAME_PROP.len());
            duk_push_lstring(self.ctx, func_name.as_ptr() as *const c_char, func_name.len());
            duk_def_prop(self.ctx, -3, (DukDefpropFlags::DUK_DEFPROP_HAVE_VALUE | DukDefpropFlags::DUK_DEFPROP_FORCE).bits());
            duk_push_lstring(self.ctx, TARGET_PROP.as_ptr() as *const c_char, TARGET_PROP.len());
            let n = size_of::<&mut dyn CallJs>();
            let p = duk_push_buffer_raw(self.ctx, n, 0);
            memcpy(transmute(p), transmute(&target), n);
            duk_def_prop(self.ctx, -3, (DukDefpropFlags::DUK_DEFPROP_HAVE_VALUE | DukDefpropFlags::DUK_DEFPROP_FORCE).bits());
            duk_put_prop_lstring(self.ctx, obj_index, func_name.as_ptr() as *const c_char, func_name.len());
        }
    }

    #[inline]
    pub fn is_string(&mut self, index: i32) -> bool {
        unsafe { duk_is_string(self.ctx, index) == 1 }
    }

    #[inline]
    pub fn is_number(&mut self, index: i32) -> bool {
        unsafe { duk_is_number(self.ctx, index) == 1 }
    }

    #[inline]
    pub fn is_object(&mut self, index: i32) -> bool {
        unsafe { duk_is_object(self.ctx, index) == 1 }
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
    pub fn get_number(&mut self, index: i32) -> f64 {
        unsafe { duk_get_number(self.ctx, index) }
    }

    #[inline]
    pub fn get_prop(&mut self, obj_index: i32) {
        unsafe {
            if duk_get_prop(self.ctx, obj_index) != 1 {
                panic!(); //FIXME (jc)
            }
        }
    }

    #[inline]
    pub fn put_prop(&mut self, obj_index: i32) {
        unsafe {
            duk_put_prop(self.ctx, obj_index);
        }
    }

    #[inline]
    pub fn get_prop_string(&mut self, obj_index: i32, key: &str) {
        unsafe {
            if duk_get_prop_lstring(self.ctx,
                                    obj_index,
                                    key.as_ptr() as *const c_char,
                                    key.len()) != 1 {
                panic!(); //FIXME (jc)
            }
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
    pub fn get_prop_index(&mut self, obj_index: i32, index: u32) {
        unsafe {
            if duk_get_prop_index(self.ctx, obj_index, index) != 1 {
                panic!(); //FIXME (jc)
            }
        }
    }

    #[inline]
    pub fn put_prop_index(&mut self, obj_index: i32, index: u32) {
        unsafe {
            duk_put_prop_index(self.ctx, obj_index, index);
        }
    }

    #[inline]
    pub fn get_global_string(&mut self, key: &str) {
        unsafe {
            duk_get_global_lstring(self.ctx, key.as_ptr() as *const c_char, key.len());
        }
    }

    #[inline]
    pub fn put_global_string(&mut self, key: &str) {
        unsafe {
            duk_put_global_lstring(self.ctx, key.as_ptr() as *const c_char, key.len());
        }
    }

    #[inline]
    pub fn call_prop(&mut self, obj_index: i32, nargs: usize) {
        unsafe {
            duk_call_prop(self.ctx, obj_index, nargs as i32);
        }
    }

    #[inline]
    pub fn eval(&mut self, filename: &str, code: &str) {
        unsafe {
            duk_push_lstring(self.ctx, filename.as_ptr() as *const c_char, filename.len());
            if duk_eval_raw(self.ctx,
                            code.as_ptr() as *const c_char,
                            code.len(),
                            1 | (DukCompileFlags::DUK_COMPILE_SAFE | DukCompileFlags::DUK_COMPILE_NOSOURCE).bits()) != 0 {
                let mut len: usize = 0;
                println!("ERR: {}",
                         CStr::from_ptr(duk_safe_to_lstring(self.ctx, -1, &mut len))
                             .to_str()
                             .unwrap()); //FIXME (jc)
                duk_pop(self.ctx);
            }
        }
    }

    //FIXME (jc) error handling
    #[inline]
    pub fn compile(&mut self, filename: &str, code: &str) {
        unsafe {
            duk_push_lstring(self.ctx, filename.as_ptr() as *const c_char, filename.len());
            if duk_compile_raw(self.ctx,
                               code.as_ptr() as *const c_char,
                               code.len(),
                               1 | DukCompileFlags::DUK_COMPILE_NORESULT.bits()) != 0 {
                let mut len: usize = 0;
                println!("ERR: {}",
                         CStr::from_ptr(duk_safe_to_lstring(self.ctx, -1, &mut len))
                             .to_str()
                             .unwrap()); //FIXME (jc)
                duk_pop(self.ctx);
            }
        }
    }

    #[inline]
    pub fn write<O: WriteJs>(&mut self, obj: &O) -> i32 {
        obj.write_js(self)
    }

    #[inline]
    pub fn read<O: ReadJs>(&mut self, obj: &mut O, obj_index: i32) {
        let obj_index = self.normalize_index(obj_index);
        obj.read_js(self, obj_index);
    }

    #[inline]
    pub fn read_top<O: ReadJs>(&mut self, obj: &mut O) {
        self.read(obj, -1);
    }

    pub fn read_node<'a>(&mut self, obj_index: i32) -> NodeRef {
        unsafe fn read<'a>(ctx: *mut duk_context, obj_index: i32) -> NodeRef {
            use self::DukType::*;
            use std::str;
            use std::slice;

            let obj_index = duk_normalize_index(ctx, obj_index);

            match DukType::from(duk_get_type(ctx, obj_index)) {
                DUK_TYPE_UNDEFINED | DUK_TYPE_NULL => NodeRef::null(),
                DUK_TYPE_BOOLEAN => NodeRef::boolean(duk_get_boolean(ctx, obj_index) == 1),
                DUK_TYPE_NUMBER => {
                    let n = duk_get_number(ctx, obj_index);
                    if n.is_normal() && (n.trunc() - n).abs() < std::f64::EPSILON {
                        NodeRef::integer(n as i64)
                    } else {
                        NodeRef::float(n)
                    }
                }
                DUK_TYPE_STRING => {
                    let mut len: usize = 0;
                    let ptr = duk_get_lstring(ctx, obj_index, Some(&mut len)) as *const u8;
                    NodeRef::string(str::from_utf8_unchecked(slice::from_raw_parts(ptr, len)))
                }
                DUK_TYPE_BUFFER => {
                    let mut len: usize = 0;
                    let ptr = duk_get_buffer(ctx, obj_index, Some(&mut len)) as *const u8;
                    NodeRef::binary(slice::from_raw_parts(ptr, len))
                }
                DUK_TYPE_OBJECT => {
                    if duk_is_array(ctx, obj_index) == 1 {
                        let len = duk_get_length(ctx, obj_index);
                        let mut elems = Elements::with_capacity(len);
                        duk_enum(ctx, obj_index, DukEnumFlags::DUK_ENUM_ARRAY_INDICES_ONLY.bits());
                        while duk_next(ctx, -1, 1) == 1 {
                            let index = duk_to_number(ctx, -2) as usize;
                            let value = read(ctx, -1);
                            if index != elems.len() {
                                panic!(); //FIXME (jc)
                            }
                            elems.push(value);
                            duk_pop_2(ctx);
                        }
                        duk_pop(ctx);
                        NodeRef::array(elems)
                    } else if duk_is_function(ctx, obj_index) == 0 && duk_is_thread(ctx, obj_index) == 0 {
                        let mut props = Properties::new();
                        duk_enum(ctx, obj_index, DukEnumFlags::DUK_ENUM_OWN_PROPERTIES_ONLY.bits());
                        while duk_next(ctx, -1, 1) == 1 {
                            let mut key_len = 0;
                            let key_ptr = duk_get_lstring(ctx, -2, Some(&mut key_len)) as *const u8;
                            let value = read(ctx, -1);
                            props.insert(str::from_utf8_unchecked(slice::from_raw_parts(key_ptr, key_len)).into(), value);
                            duk_pop_2(ctx);
                        }
                        duk_pop(ctx);
                        NodeRef::object(props)
                    } else {
                        panic!(); //FIXME (jc)
                    }
                }
                _ => panic!(), //FIXME (jc)
            }
        }

        unsafe {
            read(self.ctx, obj_index)
        }
    }

    pub fn write_node(&mut self, node: &NodeRef) -> i32 {
        unsafe fn write(ctx: *mut duk_context, data: &Node) {
            match *data.value() {
                Value::Null => {
                    duk_push_null(ctx);
                }
                Value::Boolean(b) => {
                    duk_push_boolean(ctx, b as i32);
                }
                Value::Integer(n) => {
                    duk_push_number(ctx, n as f64);
                }
                Value::Float(n) => {
                    duk_push_number(ctx, n);
                }
                Value::String(ref s) => {
                    duk_push_lstring(ctx, s.as_ptr() as *const c_char, s.len());
                }
                Value::Binary(ref b) => {
                    let p = duk_push_buffer_raw(ctx, b.len(), 1);
                    memcpy(p as *mut u8, b.as_ptr(), b.len());
                }
                Value::Array(ref elems) => {
                    let arr = duk_push_array(ctx);
                    for (i, e) in elems.iter().enumerate() {
                        write(ctx, &e.data());
                        duk_put_prop_index(ctx, arr, i as u32);
                    }
                }
                Value::Object(ref props) => {
                    let obj = duk_push_object(ctx);
                    for (k, e) in props.iter() {
                        write(ctx, &e.data());
                        let k = k.as_ref();
                        duk_put_prop_lstring(ctx, obj, k.as_ptr() as *const c_char, k.len());
                    }
                }
            }
        }

        unsafe {
            let n = node.data();
            write(self.ctx, &n);
            duk_normalize_index(self.ctx, -1)
        }
    }

}

impl Drop for Engine {
    fn drop(&mut self) {
        if self.owner && !self.ctx.is_null() {
            unsafe {
                debug!("Destroying duktape context: {:p}", self.ctx); //FIXME (jc)
                duk_destroy_heap(self.ctx);
                self.ctx = ptr::null_mut();
            }
        }
    }
}


// FIXME (jc) add error handling (methods should return Result)
pub trait ReadJs {
    fn read_js(&mut self, engine: &mut Engine, obj_index: i32);

    fn read_js_top(&mut self, engine: &mut Engine) {
        let idx = engine.normalize_index(-1);
        self.read_js(engine, idx);
    }
}


// FIXME (jc) add error handling (methods should return Result)
pub trait WriteJs {
    fn write_js(&self, engine: &mut Engine) -> i32;
}


pub trait CallJs {
    fn call(&mut self, engine: &mut Engine, func_name: &str) -> i32;
}

impl WriteJs for kg_diag::Position {
    fn write_js(&self, e: &mut Engine) -> i32 {
        let idx = e.push_object();
        e.push_number(self.offset as f64);
        e.put_prop_string(idx, "offset");
        e.push_number(self.line as f64);
        e.put_prop_string(idx, "line");
        e.push_number(self.column as f64);
        e.put_prop_string(idx, "column");
        idx
    }
}

impl ReadJs for kg_diag::Position {
    fn read_js(&mut self, e: &mut Engine, obj_index: i32) {
        e.get_prop_string(obj_index, "offset");
        self.offset = e.get_number(-1) as usize;
        e.get_prop_string(obj_index, "line");
        self.line = e.get_number(-1) as u32;
        e.get_prop_string(obj_index, "column");
        self.column = e.get_number(-1) as u32;
        e.pop_n(3);
    }
}

