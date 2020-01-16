#include <jit/jit.h>
#include <jit/jit-dump.h>
#include <stdio.h>

jit_label_t emptylbl() {
    return ((jit_label_t)~((jit_uint)0));
};

void printfunc(jit_function_t func) {
    jit_dump_function(stdout, func, "dumpfunc");
    putchar('\n');
    fflush(stdout);
};

void printtype(jit_type_t tp) {
    jit_dump_type(stdout, tp);
    putchar('\n');
    fflush(stdout);
};

void printval(jit_function_t func, jit_value_t val) {
    jit_dump_value(stdout, func, val, NULL);
    putchar('\n');
    fflush(stdout);
}