use std::ops::DerefMut;
use super::*;

macro_rules! try_exec_success {
    ($res:expr) => {
        if $res != DUK_EXEC_SUCCESS {
            return Err($res)
        }
    }
}

/// Wrapper for Duktape context
#[derive(Debug)]
pub struct DukContext {
    pub (crate) ctx: *mut duk_context,
}

impl DukContext {
    pub (crate) unsafe fn from_raw(ctx: *mut duk_context) -> Self {
        Self { ctx }
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
    pub fn dup(&self, index: i32) {
        unsafe {
            duk_dup(self.ctx, index);
        }
    }

    #[inline]
    pub fn remove(&self, index: i32) {
        unsafe {
            duk_remove(self.ctx, index);
        }
    }

    #[inline]
    pub fn pop(&self) {
        unsafe {
            duk_pop(self.ctx);
        }
    }

    #[inline]
    pub fn pop_n(&self, n: i32) {
        unsafe {
            duk_pop_n(self.ctx, n);
        }
    }

    #[inline]
    pub fn swap(&self, idx1: i32, idx2: i32) {
        unsafe {
            duk_swap(self.ctx, idx1, idx2);
        }
    }

    #[inline]
    pub fn push_this(&self) {
        unsafe { duk_push_this(self.ctx); }
    }

    #[inline]
    pub fn push_thread(&self) -> i32 {
        unsafe { duk_push_thread_raw(self.ctx, 0) }
    }

    #[inline]
    pub fn push_thread_new_globalenv(&self) -> i32 {
        unsafe { duk_push_thread_raw(self.ctx, DukThreadFlags::DUK_THREAD_NEW_GLOBAL_ENV.bits()) }
    }

    #[inline]
    pub fn push_global_object(&self) {
        unsafe { duk_push_global_object(self.ctx); }
    }

    #[inline]
    pub fn push_boolean(&self, value: bool) {
        unsafe { duk_push_boolean(self.ctx, value as i32) }
    }

    #[inline]
    pub fn push_null(&self) {
        unsafe { duk_push_null(self.ctx) }
    }

    #[inline]
    pub fn push_undefined(&self) {
        unsafe { duk_push_undefined(self.ctx) }
    }

    #[inline]
    pub fn push_i32(&self, value: i32) {
        unsafe { duk_push_int(self.ctx, value) }
    }

    #[inline]
    pub fn push_u32(&self, value: u32) {
        unsafe { duk_push_uint(self.ctx, value) }
    }

    #[inline]
    pub fn push_number(&self, value: f64) {
        unsafe { duk_push_number(self.ctx, value) }
    }

    #[inline]
    pub fn push_string(&self, value: &str) {
        unsafe {
            duk_push_lstring(self.ctx, value.as_ptr() as *const c_char, value.len());
        }
    }

    #[inline]
    pub fn push_object(&self) -> i32 {
        unsafe { duk_push_object(self.ctx) }
    }

    #[inline]
    pub fn push_ext_buffer(&self, data: &[u8]) {
        unsafe {
            duk_push_buffer_raw(self.ctx, 0, (DukBufFlags::DUK_BUF_FLAG_DYNAMIC | DukBufFlags::DUK_BUF_FLAG_EXTERNAL).bits());
            duk_config_buffer(self.ctx, -1, data.as_ptr() as *mut c_void, data.len());
        }
    }

    #[inline]
    pub fn push_array(&self) -> i32 {
        unsafe { duk_push_array(self.ctx) }
    }

    pub fn push_function(&self, func_name: &str, nargs: i32) {
        unsafe {
            duk_push_c_function(self.ctx, Some(func_dispatch), nargs);
            duk_push_lstring(self.ctx, FUNC_NAME_PROP.as_ptr() as *const c_char, FUNC_NAME_PROP.len());
            duk_push_lstring(self.ctx, func_name.as_ptr() as *const c_char, func_name.len());
            duk_def_prop(self.ctx, -3, (DukDefpropFlags::DUK_DEFPROP_ENUMERABLE | DukDefpropFlags::DUK_DEFPROP_HAVE_VALUE).bits())
        }
    }

    pub fn put_prop_function(&self, obj_index: i32, func_name: &str, nargs: i32) {
        let obj_index = self.normalize_index(obj_index);
        self.push_function(func_name, nargs);
        unsafe {
            duk_put_prop_lstring(self.ctx, obj_index, func_name.as_ptr() as *const c_char, func_name.len());
        }
    }

    pub fn put_global_function(&self, func_name: &str, nargs: i32) {
        self.push_function(func_name, nargs);
        self.put_global_string(func_name);
    }

    #[inline]
    pub fn get_type(&self, index: i32) -> DukType {
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
    pub fn get_string(&self, index: i32) -> &str {
        use std::str;
        use std::slice;
        unsafe {
            let mut len: usize = 0;
            let ptr = duk_get_lstring(self.ctx, index, Some(&mut len)) as *const u8;
            str::from_utf8_unchecked(slice::from_raw_parts(ptr, len))
        }
    }

    #[inline]
    pub fn get_buffer(&self, index: i32) -> &[u8] {
        use std::slice;
        unsafe {
            let mut len: usize = 0;
            let ptr = duk_get_buffer(self.ctx, index, Some(&mut len)) as *const u8;
            slice::from_raw_parts(ptr, len)
        }
    }

    #[inline]
    pub fn get_number(&self, index: i32) -> f64 {
        unsafe { duk_get_number(self.ctx, index) }
    }

    #[inline]
    pub fn get_boolean(&self, index: i32) -> bool {
        unsafe { duk_get_boolean(self.ctx, index) != 0 }
    }

    pub fn get_context(&self, index: i32) -> Result<DukContextGuard, JsError> {
        let new_ctx = unsafe { duk_get_context(self.ctx, index) };
        if new_ctx.is_null() {
            return Err(JsError::from(format!("could not get context from index {}", index)));
        }
        Ok(DukContextGuard::new(unsafe { DukContext::from_raw(new_ctx) }))
    }

    #[inline]
    pub fn get_prop(&self, obj_index: i32) -> bool {
        unsafe { duk_get_prop(self.ctx, obj_index) == 1 }
    }

    #[inline]
    pub fn put_prop(&self, obj_index: i32) {
        unsafe { duk_put_prop(self.ctx, obj_index); }
    }

    #[inline]
    pub fn get_prop_string(&self, obj_index: i32, key: &str) -> bool {
        unsafe {
            duk_get_prop_lstring(self.ctx, obj_index, key.as_ptr() as *const c_char, key.len()) == 1
        }
    }

    #[inline]
    pub fn put_prop_string(&self, obj_index: i32, key: &str) {
        unsafe {
            duk_put_prop_lstring(self.ctx,
                                 obj_index,
                                 key.as_ptr() as *const c_char,
                                 key.len());
        }
    }

    #[inline]
    pub fn get_prop_index(&self, obj_index: i32, index: u32) -> bool {
        unsafe { duk_get_prop_index(self.ctx, obj_index, index) == 1 }
    }

    #[inline]
    pub fn put_prop_index(&self, obj_index: i32, index: u32) {
        unsafe {
            duk_put_prop_index(self.ctx, obj_index, index);
        }
    }

    #[inline]
    pub fn get_global_string(&self, key: &str) -> bool {
        unsafe {
            duk_get_global_lstring(self.ctx, key.as_ptr() as *const c_char, key.len()) == 1
        }
    }

    #[inline]
    pub fn put_global_string(&self, key: &str) {
        unsafe {
            duk_put_global_lstring(self.ctx, key.as_ptr() as *const c_char, key.len());
        }
    }

    #[inline]
    pub fn get_length(&self, obj_index: i32) -> usize {
        unsafe {
            duk_get_length(self.ctx, obj_index)
        }
    }

    #[inline]
    pub fn enum_indices(&self, obj_index: i32) {
        unsafe {
            duk_enum(self.ctx, obj_index, DukEnumFlags::DUK_ENUM_ARRAY_INDICES_ONLY.bits());
        }
    }

    #[inline]
    pub fn enum_keys(&self, obj_index: i32) {
        unsafe {
            duk_enum(self.ctx, obj_index, DukEnumFlags::DUK_ENUM_OWN_PROPERTIES_ONLY.bits());
        }
    }

    #[inline]
    pub fn next(&self, obj_index: i32) -> bool {
        unsafe {
            duk_next(self.ctx, obj_index, 1) == 1
        }
    }

    #[inline]
    pub fn call_prop(&self, obj_index: i32, nargs: usize) {
        unsafe {
            duk_call_prop(self.ctx, obj_index, nargs as i32);
        }
    }

    #[inline]
    pub fn pcall(&self, nargs: usize) -> Result<(), i32> {
        let res = unsafe {
            duk_pcall(self.ctx, nargs as i32)
        };
        try_exec_success!(res);
        Ok(())
    }

    #[inline]
    pub fn pcall_method(&self, nargs: usize) -> Result<(), i32> {
        let res = unsafe {
            duk_pcall_method(self.ctx, nargs as i32)
        };
        try_exec_success!(res);
        Ok(())
    }

    #[inline]
    pub fn pcall_prop(&self, obj_index: i32, nargs: usize) -> Result<(), i32> {
        let res = unsafe {
            duk_pcall_prop(self.ctx, obj_index, nargs as i32)
        };
        try_exec_success!(res);
        Ok(())
    }

    #[inline]
    pub fn safe_to_lstring(&self, obj_index: i32) -> String {
        unsafe {
            let mut len: usize = 0;
            let msg = duk_safe_to_lstring(self.ctx, obj_index, &mut len);
            String::from(std::str::from_utf8_unchecked(std::slice::from_raw_parts(msg as *const u8, len)))
        }
    }

    #[inline]
    pub fn throw(&self) {
        unsafe {
            duk_throw_raw(self.ctx);
        }
    }

    #[inline]
    pub fn push_context_dump(&self) {
        unsafe {
            duk_push_context_dump(self.ctx);
        }
    }

    pub fn get_stack_dump(&self) -> String {
        self.push_context_dump();
        unsafe {
            let dump = CStr::from_ptr(duk_to_string(self.ctx, -1)).to_string_lossy().to_string();
            duk_pop(self.ctx);
            dump
        }
    }

    /// Propagate JS error to Rust, popping the error from the stack.
    /// js_res: Result<(), i32> - JS result returned by protected call functions.
    /// If it is an error, it will be converted to JsError.
    /// This method should be called immediately after a protected call to handle the error.
    pub fn propagate_js_error<T>(&self, js_res: Result<T, i32>) -> Result<T, JsError> {
        unsafe {
            match js_res {
                Ok(v) => Ok(v),
                Err(_err) => {
                    let mut len: usize = 0;
                    let msg = duk_safe_to_lstring(self.ctx, -1, &mut len);
                    let s = String::from(std::str::from_utf8_unchecked(std::slice::from_raw_parts(msg as *const u8, len)));
                    duk_pop(self.ctx);
                    Err(JsError::from(s))
                }
            }
        }
    }

    #[inline]
    pub fn eval(&self, code: &str) -> Result<(), JsError> {
        unsafe {
            if duk_eval_raw(self.ctx,
                            code.as_ptr() as *const c_char,
                            code.len(),
                            0 | (DukCompileFlags::DUK_COMPILE_SAFE | DukCompileFlags::DUK_COMPILE_NOSOURCE | DukCompileFlags::DUK_COMPILE_NOFILENAME).bits()) != 0 {
                let mut len: usize = 0;
                let msg = duk_safe_to_lstring(self.ctx, -1, &mut len);
                let s = String::from(std::str::from_utf8_unchecked(std::slice::from_raw_parts(msg as *const u8, len)));
                duk_pop(self.ctx);
                Err(JsError::from(s))
            } else {
                Ok(())
            }
        }
    }

    #[inline]
    pub fn eval_file(&self, filename: &str, code: &str) -> Result<(), JsError> {
        unsafe {
            duk_push_lstring(self.ctx, filename.as_ptr() as *const c_char, filename.len());
            if duk_eval_raw(self.ctx,
                            code.as_ptr() as *const c_char,
                            code.len(),
                            1 | (DukCompileFlags::DUK_COMPILE_SAFE | DukCompileFlags::DUK_COMPILE_NOSOURCE).bits()) != 0 {
                let s = self.safe_to_lstring(-1);
                duk_pop(self.ctx);
                Err(JsError::from(s))
            } else {
                Ok(())
            }
        }
    }

    #[inline]
    pub fn compile(&self, code: &str) -> Result<(), JsError> {
        unsafe {
            if duk_compile_raw(self.ctx,
                               code.as_ptr() as *const c_char,
                               code.len(),
                               0 | (DukCompileFlags::DUK_COMPILE_NORESULT | DukCompileFlags::DUK_COMPILE_NOFILENAME).bits()) != 0 {
                let s = self.safe_to_lstring(-1);
                duk_pop(self.ctx);
                Err(JsError::from(s))
            } else {
                Ok(())
            }
        }
    }

    #[inline]
    pub fn compile_file(&self, filename: &str, code: &str) -> Result<(), JsError> {
        unsafe {
            duk_push_lstring(self.ctx, filename.as_ptr() as *const c_char, filename.len());
            if duk_compile_raw(self.ctx,
                               code.as_ptr() as *const c_char,
                               code.len(),
                               1 | DukCompileFlags::DUK_COMPILE_NORESULT.bits()) != 0 {
                let s = self.safe_to_lstring(-1);
                duk_pop(self.ctx);
                Err(JsError::from(s))
            } else {
                Ok(())
            }
        }
    }

    #[inline]
    pub fn write<O: WriteJs>(&self, obj: &O) -> Result<(), JsError> {
        obj.write_js(self)
    }

    #[inline]
    pub fn read<O: ReadJs>(&self, obj_index: i32) -> Result<O, JsError> {
        let obj_index = self.normalize_index(obj_index);
        O::read_js(self, obj_index)
    }

    #[inline]
    pub fn read_top<O: ReadJs>(&self) -> Result<O, JsError> {
        self.read( -1)
    }

    /// Initialize console functions.
    #[inline]
    pub fn init_console(&self) {
        unsafe {
            duk_api_console_init(self.ctx, Some(console_func));
        }
    }

    #[inline]
    pub fn xcopy_top(&self, from: &DukContext, count: i32) {
        unsafe {
            duk_xcopymove_raw(self.ctx, from.ctx, count, 1);
        }
    }

    #[inline]
    pub fn xmove_top(&self, from: &mut DukContext, count: i32) {
        unsafe {
            duk_xcopymove_raw(self.ctx, from.ctx, count, 0);
        }
    }

    #[inline]
    pub fn check_stack(&self, extra: i32) -> Result<(), JsError> {
        let res = unsafe {
            duk_check_stack(self.ctx, extra)
        };

        if res {
            Ok(())
        } else {
            Err(JsError::from("failed to reserve enough stack space".to_string()))
        }
    }

    #[inline]
    pub fn check_stack_top(&self, top: i32) -> Result<(), JsError> {
        let res = unsafe {
            duk_check_stack_top(self.ctx, top)
        };
        if res {
            Ok(())
        } else {
            Err(JsError::from("failed to reserve enough stack space".to_string()))
        }
    }

    #[inline]
    pub fn set_global_object(&self) {
        unsafe {
            duk_set_global_object(self.ctx);
        }
    }

    pub fn gc(&self) {
        unsafe {
            duk_gc(self.ctx, DukGcFlags::NONE.bits());
        }
    }
}

pub struct DukContextGuard<'a> {
    ctx: DukContext,
    _marker: std::marker::PhantomData<&'a JsEngine>,
}

impl std::fmt::Debug for DukContextGuard<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DukContextGuard").finish()
    }
}

impl Deref for DukContextGuard<'_> {
    type Target = DukContext;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl DerefMut for DukContextGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}

impl <'a> DukContextGuard<'a> {
    pub fn new(ctx: DukContext) -> Self {
        Self {
            ctx,
            _marker: std::marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use super::*;

    #[test]
    fn test_eval() {
        let engine = JsEngine::new().unwrap();
        //language=js
        engine.eval(r#" var tmp =  {
            "foo": 1,
            "bar": "baz"
        }
        tmp
        "#).unwrap();

        #[derive(Deserialize)]
        struct TestStruct {
            foo: i32,
        }
        assert_eq!(engine.read_top::<TestStruct>().unwrap().foo, 1);
        engine.pop();
    }

    #[test]
    fn test_get_invalid_context() {
        let engine = JsEngine::new().unwrap();
        let res = engine.get_context(0);
        assert!(res.is_err());
    }

    #[test]
    fn test_push_thread() {
        let engine = JsEngine::new().unwrap();
        let new_idx = engine.push_thread();
        let new_ctx = engine.get_context(new_idx).unwrap();
        new_ctx.push_string("test");
        assert_eq!(new_ctx.get_string(-1), "test");
        new_ctx.pop();
        assert!(new_ctx.get_stack_dump().contains("ctx: top=0"));

        drop(new_ctx);
        engine.pop();
        assert!(engine.get_stack_dump().contains("ctx: top=0"));
    }

    #[test]
    fn test_nested_push_thread() {
        let engine = JsEngine::new().unwrap();
        let new_idx = engine.push_thread();
        let new_ctx = engine.get_context(new_idx).unwrap();

        let nested_id = new_ctx.push_thread();
        let nested_ctx = new_ctx.get_context(nested_id).unwrap();
        nested_ctx.push_string("test");

        assert_eq!(nested_ctx.get_string(-1), "test");

        drop(nested_ctx);
        drop(new_ctx);

        engine.pop();

        assert!(engine.get_stack_dump().contains("ctx: top=0"));
    }

    #[test]
    fn test_push_thread_new_globalenv() {
        let engine = JsEngine::new().unwrap();

        let new_idx = engine.push_thread_new_globalenv();
        let new_idx2 = engine.push_thread_new_globalenv();

        let new_ctx = engine.get_context(new_idx).unwrap();
        let new_ctx2 = engine.get_context(new_idx2).unwrap();

        // Test first context
        new_ctx.push_string("test");
        assert_eq!(new_ctx.get_string(-1), "test");
        new_ctx.pop();
        assert!(new_ctx.get_stack_dump().contains("ctx: top=0"));

        // Test second context
        new_ctx2.push_string("test2");
        new_ctx2.push_string("test2");
        assert_eq!(new_ctx2.get_string(-1), "test2");
        // Pop only one string
        new_ctx2.pop();
        assert!(new_ctx2.get_stack_dump().contains("ctx: top=1"));

        drop(new_ctx);
        drop(new_ctx2);

        // Pop both contexts
        engine.pop_n(2);

        assert!(engine.get_stack_dump().contains("ctx: top=0"));
    }

    #[test]
    fn test_to_lstring_safety() {
        let engine = JsEngine::new().unwrap();
        engine.push_string("test");
        let s = engine.safe_to_lstring(-1);
        assert_eq!(s, "test");
        engine.pop();
        assert_eq!(s, "test");
        drop(engine);
        assert_eq!(s, "test");
    }

    #[test]
    fn test_check_stack_error() {
        let engine = JsEngine::new().unwrap();
        let res = engine.check_stack(i32::MAX);
        assert!(res.is_err());
    }

    #[test]
    fn test_xcopy_top() {
        let engine = JsEngine::new().unwrap();

        //language=javascript
        engine.eval("GLOBAL_TEST = { a: 1, b: 2 }").unwrap();

        let ctx1_idx = engine.push_thread_new_globalenv();
        let ctx1 = engine.get_context(ctx1_idx).unwrap();
        // Create a new global variable
        assert!(engine.get_global_string("GLOBAL_TEST"));

        ctx1.check_stack(1).unwrap();
        // Copy the global variable to the new context
        ctx1.xcopy_top(&engine, 1);
        ctx1.put_global_string("GLOBAL_TEST");

        // Check if the global variable is available in the new context
        ctx1.eval("GLOBAL_TEST.b").unwrap();
        assert_eq!(ctx1.get_number(-1), 2.0);

        // Change the value of the global variable in the new context
        ctx1.eval("GLOBAL_TEST.b = 5").unwrap();

        // Check if the value has changed in original context
        engine.eval("GLOBAL_TEST.b").unwrap();
        assert_eq!(engine.get_number(-1), 5.0);
        engine.pop();
    }

    #[test]
    fn test_set_global_object() {
        let engine = JsEngine::new().unwrap();

        engine.eval("Math.abs(-1)").unwrap();
        assert_eq!(engine.get_number(-1), 1.0);

        engine.pop();

        //language=javascript
        let _: () = engine.eval("var obj = { a: 1, b: 2 }; obj").unwrap();
        engine.set_global_object();

        engine.eval("typeof Math").unwrap();

        assert_eq!(engine.get_string(-1), "undefined");

    }

    #[test]
    fn test_gc() {
        let engine = JsEngine::new().unwrap();

        engine.eval("Math.abs(-1)").unwrap();
        assert_eq!(engine.get_number(-1), 1.0);
        engine.pop();
        engine.gc();
    }

    #[test]
    fn tes_propagate_js_error() {
        let engine = JsEngine::new().unwrap();
        //language=js
        engine.eval(r#" var tmp =  {
            "foo": 1,
            "bar": "baz",
            "some_fn": function() {
                throw new Error("test error");
            }
        }
        tmp
        "#).unwrap();

        engine.push_string("some_fn");

        let call_res = engine.pcall_prop(0, 0);
        assert!(call_res.is_err());
        let res = engine.propagate_js_error(call_res);

        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.to_string().contains("test error"));
    }
}

