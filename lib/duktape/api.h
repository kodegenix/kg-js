#pragma once

#include "duktape.h"

typedef void (*duk_console_function) (void* udata, duk_uint_t fun, const char *msg, duk_size_t msg_len);

extern unsigned int duk_api_version();
extern const char* duk_api_git_commit();
extern const char* duk_api_git_describe();
extern const char* duk_api_git_branch();

extern void* duk_api_get_heap_udata(duk_context* ctx);

extern void duk_api_console_init(duk_context *ctx, duk_console_function console_cb);
