#include <jit/jit.h>
#include <jit/jit-dump.h>
#include <stdio.h>

jit_label_t emptylbl() {
    return ((jit_label_t)~((jit_uint)0));
};

void printfunc(jit_function_t func) {
    jit_dump_function(stdout, func, "dumpfunc");
};