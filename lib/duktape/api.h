#include "duktape.h"
#include "duk_console.h"

extern unsigned int duk_version();
extern const char* duk_git_commit();
extern const char* duk_git_describe();
extern const char* duk_git_branch();

extern duk_context* duk_create_context(void *heap_udata, duk_fatal_function fatal_handler);
