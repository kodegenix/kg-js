#include "api.h"

unsigned int duk_api_version() {
    return DUK_VERSION;
}

const char* duk_api_git_commit() {
    return DUK_GIT_COMMIT;
}

const char* duk_api_git_describe() {
    return DUK_GIT_DESCRIBE;
}

const char* duk_api_git_branch() {
    return DUK_GIT_BRANCH;
}

void* duk_api_get_heap_udata(duk_context* ctx) {
    duk_memory_functions func;
    duk_get_memory_functions(ctx, &func);
    return func.udata;
}


static duk_ret_t duk__console_log_helper(duk_context *ctx, const char *error_name) {
    duk_int_t fun = duk_get_current_magic(ctx);

    duk_idx_t n = duk_get_top(ctx);
    duk_idx_t i;

    duk_get_global_string(ctx, "console");
    duk_get_prop_string(ctx, -1, DUK_HIDDEN_SYMBOL("console_callback"));
    duk_console_function callback = duk_require_pointer(ctx, -1);
    duk_pop(ctx);
    duk_get_prop_string(ctx, -1, "format");

    for (i = 0; i < n; i++) {
        if (duk_check_type_mask(ctx, i, DUK_TYPE_MASK_OBJECT)) {
            /* Slow path formatting. */
            duk_dup(ctx, -1);  /* console.format */
            duk_dup(ctx, i);
            duk_call(ctx, 1);
            duk_replace(ctx, i);  /* arg[i] = console.format(arg[i]); */
        }
    }

    duk_pop_2(ctx);

    duk_push_string(ctx, " ");
    duk_insert(ctx, 0);
    duk_join(ctx, n);

    if (error_name) {
        duk_push_error_object(ctx, DUK_ERR_ERROR, "%s", duk_require_string(ctx, -1));
        duk_push_string(ctx, "name");
        duk_push_string(ctx, error_name);
        duk_def_prop(ctx, -3, DUK_DEFPROP_FORCE | DUK_DEFPROP_HAVE_VALUE);  /* to get e.g. 'Trace: 1 2 3' */
        duk_get_prop_string(ctx, -1, "stack");
    }

    duk_size_t len = 0;
    const char* msg = duk_to_lstring(ctx, -1, &len);
    void* udata = duk_api_get_heap_udata(ctx);
    callback(udata, fun, msg, len);

    return 0;
}

static duk_ret_t duk__console_assert(duk_context *ctx) {
    if (duk_to_boolean(ctx, 0)) {
        return 0;
    }
    duk_remove(ctx, 0);

    return duk__console_log_helper(ctx, "AssertionError");
}

static duk_ret_t duk__console_log(duk_context *ctx) {
    return duk__console_log_helper(ctx, NULL);
}

static duk_ret_t duk__console_trace(duk_context *ctx) {
    return duk__console_log_helper(ctx, "Trace");
}

static duk_ret_t duk__console_info(duk_context *ctx) {
    return duk__console_log_helper(ctx, NULL);
}

static duk_ret_t duk__console_warn(duk_context *ctx) {
    return duk__console_log_helper(ctx, NULL);
}

static duk_ret_t duk__console_error(duk_context *ctx) {
    return duk__console_log_helper(ctx, "Error");
}

static duk_ret_t duk__console_dir(duk_context *ctx) {
    return duk__console_log_helper(ctx, 0);
}

static void duk__console_reg_vararg_func(duk_context *ctx, duk_c_function func, const char *name, duk_uint_t flags) {
    duk_push_c_function(ctx, func, DUK_VARARGS);
    duk_push_string(ctx, "name");
    duk_push_string(ctx, name);
    duk_def_prop(ctx, -3, DUK_DEFPROP_HAVE_VALUE | DUK_DEFPROP_FORCE);  /* Improve stacktraces by displaying function name */
    duk_set_magic(ctx, -1, (duk_int_t) flags);
    duk_put_prop_string(ctx, -2, name);
}

void duk_api_console_init(duk_context *ctx, duk_console_function console_cb) {
    duk_push_object(ctx);
    duk_push_pointer(ctx, console_cb);
    duk_put_prop_string(ctx, -2, DUK_HIDDEN_SYMBOL("console_callback"));

    /* Custom function to format objects; user can replace.
     * For now, try JX-formatting and if that fails, fall back
     * to ToString(v).
     */
    duk_eval_string(ctx, "(function(E){return function format(v){try{return E('jx',v);}catch(e){return String(v);}};})(Duktape.enc)");
    duk_put_prop_string(ctx, -2, "format");

    duk__console_reg_vararg_func(ctx, duk__console_assert, "assert", 1);
    duk__console_reg_vararg_func(ctx, duk__console_log, "log", 2);
    duk__console_reg_vararg_func(ctx, duk__console_log, "debug", 3);  /* alias to console.log */
    duk__console_reg_vararg_func(ctx, duk__console_trace, "trace", 4);
    duk__console_reg_vararg_func(ctx, duk__console_info, "info", 5);
    duk__console_reg_vararg_func(ctx, duk__console_warn, "warn", 6);
    duk__console_reg_vararg_func(ctx, duk__console_error, "error", 7);
    duk__console_reg_vararg_func(ctx, duk__console_error, "exception", 8);  /* alias to console.error */
    duk__console_reg_vararg_func(ctx, duk__console_dir, "dir", 9);

    duk_put_global_string(ctx, "console");
}
