#include "api.h"

unsigned int duk_version() {
    return DUK_VERSION;
}

const char* duk_git_commit() {
    return DUK_GIT_COMMIT;
}

const char* duk_git_describe() {
    return DUK_GIT_DESCRIBE;
}

const char* duk_git_branch() {
    return DUK_GIT_BRANCH;
}


duk_context* duk_create_context(void *heap_udata, duk_fatal_function fatal_handler) {
    duk_context* ctx = duk_create_heap(NULL, NULL, NULL, heap_udata, fatal_handler);
    if (ctx == NULL) {
        return NULL;
    }
    duk_console_init(ctx, 0);
    return ctx;
}

