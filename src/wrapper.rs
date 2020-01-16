use crate::bindings::*;
use std::mem;
use std::ptr;
use libc::{c_void};
use std::convert::TryInto;

pub struct Context {
    ptr: *mut _jit_context,
    nextfunc: *mut _jit_function, // for iterator
}

#[derive(Copy, Clone)]
pub struct Function {
    pub ptr: *mut _jit_function,
    argc: u32,
}

#[derive(Copy, Clone)]
pub struct Value {
    ptr: *mut _jit_value,
}

#[derive(Copy, Clone)]
pub struct Type {
    ptr: *mut _jit_type,
}

pub type Signature = Type;

pub struct Label {
    ptr: *mut jit_label_t,
}

impl Context {
    pub fn new() -> Self {
        unsafe {
            let ctx = jit_context_create();
            jit_context_build_start(ctx);
            Context {ptr: ctx, nextfunc: ptr::null_mut()}
        }
    }

    pub fn new_function(&self, params: &[Type], ret_type: Type) -> Function {
        unsafe {
            let signature = Signature::create_signature(params, ret_type);
            Function {ptr: jit_function_create(self.ptr, signature.ptr), argc: params.len() as u32}
        }
    }

    pub fn finish(&self) {
        unsafe {
            jit_context_build_end(self.ptr);
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            jit_context_destroy(self.ptr);
        }
    }
}

impl Iterator for Context {
    type Item = Function;

    fn next(&mut self) -> Option<Function> {
        unsafe {
            let nextf = jit_function_next(self.ptr, self.nextfunc);
            self.nextfunc = nextf;
            if nextf == ptr::null_mut() {
                None
            } else {
                Some(Function {ptr: nextf, argc: jit_type_num_params(jit_function_get_signature(nextf))})
            }
        }
    }
}

impl Function {
    pub fn get_params(&self) -> Vec<Value> {
        unsafe {
            let mut params = Vec::new();
            for i in 0..self.argc {
                params.push(Value::new(jit_value_get_param(self.ptr, i as u32)));
            }
            params
        }
    }

    pub fn dump(&self) {
        unsafe {
            printfunc(self.ptr);
        }
    }

    pub fn optimize(&self) {
        unsafe {
            jit_function_set_optimization_level(self.ptr, 1);
            jit_optimize(self.ptr);
        }
    }

    pub fn compile(&self) -> i32 {
        unsafe {
            jit_function_set_optimization_level(self.ptr, jit_function_get_max_optimization_level());
            jit_function_compile(self.ptr)
        }
    }

    // execute the function without args and return i32 ('main' signature)
    pub fn standard_execute(&self) -> i32 {
        unsafe {
            let mut dummy = 0;
            let mut args : [*mut c_void; 1] = [mem::transmute(&mut dummy)];
            let mut res : i32 = 0;
            jit_function_apply(self.ptr, args.as_mut_ptr(), &mut res as *mut i32 as *mut c_void);
            res
        }
    }

    pub fn i_add(&self, val1: &Value, val2: &Value) -> Value {
        unsafe {
            Value::new(jit_insn_add(self.ptr, val1.ptr, val2.ptr))
        }
    }

    pub fn i_sub(&self, val1: &Value, val2: &Value) -> Value {
        unsafe {
            Value::new(jit_insn_sub(self.ptr, val1.ptr, val2.ptr))
        }
    }

    pub fn i_mul(&self, val1: &Value, val2: &Value) -> Value {
        unsafe {
            Value::new(jit_insn_mul(self.ptr, val1.ptr, val2.ptr))
        }
    }

    pub fn i_eq(&self, val1: &Value, val2: &Value) -> Value {
        unsafe {
            Value::new(jit_insn_eq(self.ptr, val1.ptr, val2.ptr))
        }
    }

    pub fn i_ne(&self, val1: &Value, val2: &Value) -> Value {
        unsafe {
            Value::new(jit_insn_ne(self.ptr, val1.ptr, val2.ptr))
        }
    }

    pub fn i_lt(&self, val1: &Value, val2: &Value) -> Value {
        unsafe {
            Value::new(jit_insn_lt(self.ptr, val1.ptr, val2.ptr))
        }
    }

    pub fn i_le(&self, val1: &Value, val2: &Value) -> Value {
        unsafe {
            Value::new(jit_insn_le(self.ptr, val1.ptr, val2.ptr))
        }
    }

    pub fn i_gt(&self, val1: &Value, val2: &Value) -> Value {
        unsafe {
            Value::new(jit_insn_gt(self.ptr, val1.ptr, val2.ptr))
        }
    }

    pub fn i_ge(&self, val1: &Value, val2: &Value) -> Value {
        unsafe {
            Value::new(jit_insn_ge(self.ptr, val1.ptr, val2.ptr))
        }
    }

    pub fn i_and(&self, val1: &Value, val2: &Value) -> Value {
        unsafe {
            Value::new(jit_insn_and(self.ptr, val1.ptr, val2.ptr))
        }
    }

    pub fn i_or(&self, val1: &Value, val2: &Value) -> Value {
        unsafe {
            Value::new(jit_insn_or(self.ptr, val1.ptr, val2.ptr))
        }
    }

    pub fn i_convert(&self, val: &Value, tp: Type) -> Value {
        unsafe {
            Value::new(jit_insn_convert(self.ptr, val.ptr, tp.ptr, 0))
        }
    }

    pub fn i_normal_call(&self, f: &Function, args: &[Value]) -> Value {
        unsafe {
            let mut argsval : Vec<*mut _jit_value> = Vec::with_capacity(args.len());
            for a in args {
                argsval.push(a.ptr);
            }
            Value::new(jit_insn_call(self.ptr, ptr::null(), f.ptr, ptr::null_mut(), argsval.as_mut_ptr(), args.len().try_into().unwrap(), 0))
        }
    }

    // return type is needed as it cannot be inferred
    pub fn i_native_call(&self, f: *mut c_void, args: &[Value], ret_type: Type) -> Value {
        unsafe {
            let mut argsval: Vec<*mut _jit_value> = Vec::with_capacity(args.len());
            let mut types : Vec<Type> = Vec::with_capacity(args.len());
            for a in args {
                types.push(a.get_type());
                argsval.push(a.ptr);
            }
            // constructor signature from values (assume correct types passed in)
            let signature = Signature::create_signature(types.as_slice(), ret_type);
            Value::new(jit_insn_call_native(self.ptr, ptr::null(), f, signature.ptr, argsval.as_mut_ptr(), args.len().try_into().unwrap(), 0))
        }
    }

    pub fn i_branch_if(&self, val: &Value, brnch: &Label) {
        unsafe {
            jit_insn_branch_if(self.ptr, val.ptr, brnch.ptr);
        }
    }

    pub fn i_branch_if_not(&self, val: &Value, brnch: &Label) {
        unsafe {
            jit_insn_branch_if_not(self.ptr, val.ptr, brnch.ptr);
        }
    }

    pub fn i_return(&self, val: &Value) {
        unsafe {
            jit_insn_return(self.ptr, val.ptr);
        }
    }

    pub fn i_alloca(&self, size: i64) -> Value {
        unsafe {
            Value::new(jit_insn_alloca(self.ptr, Value::constant_long(&self, size).ptr))
        }
    }

    pub fn i_store(&self, val: &Value, dest: &Value) {
        unsafe {
            jit_insn_store(self.ptr, dest.ptr, val.ptr);
        }
    }

    pub fn i_load(&self, dest: &Value) -> Value {
        unsafe {
            Value::new(jit_insn_load(self.ptr, dest.ptr))
        }
    }
}

impl Value {

    fn new(ptr: *mut _jit_value) -> Self {
        Value {ptr}
    }

    pub fn create(func: &Function, tp: &Type) -> Self {
        unsafe {
            Value::new(jit_value_create(func.ptr, tp.ptr))
        }
    }

    pub fn constant(func: &Function, tp: Type, val: i64) -> Self {
        unsafe {
            Value::new(jit_value_create_nint_constant(func.ptr, tp.ptr, val))
        }
    }

    pub fn constant_long(func: &Function, val: i64) -> Self {
        Value::constant(func, Type::int(), val)
    }

    pub fn constant_void(func: &Function) -> Self {
        Value::constant(func, Type::void(), 0)
    }

    pub fn get_type(&self) -> Type {
        unsafe {
            Type {ptr: jit_value_get_type(self.ptr)}
        }
    }

    pub fn dump(&self, f: &Function) {
        unsafe {
            printval(f.ptr, self.ptr);
        }
    }

}

impl Label {
    pub fn new() -> Self {
        unsafe {
            let lbl = Box::new(emptylbl());
            Label {ptr: Box::into_raw(lbl)}
        }
    }
    
    pub fn place(&self, f: &Function) {
        unsafe {
            jit_insn_label(f.ptr, self.ptr);
        }
    }
}

impl Type {

    pub fn create_signature(params: &[Type], ret: Type) -> Signature {
        unsafe {
            let size = params.len() as u32;
            let mut types : Vec<jit_type_t> = Vec::with_capacity(params.len());
            for p in params {
                types.push(p.ptr)
            };
            Signature {ptr: jit_type_create_signature(
                jit_abi_t_jit_abi_cdecl,
                ret.ptr, 
                types.as_mut_ptr(), 
                size, 0
            )}
        }
    }

    pub fn pointer(pointed_type: &Type) -> Self {
        unsafe {
            Type{ptr: jit_type_create_pointer(pointed_type.ptr, 0)}
        }
    }

    pub fn int() -> Self {
        unsafe {
            Type {ptr: jit_type_nint}
        }
    }

    pub fn bool() -> Self {
        unsafe {
            Type {ptr: jit_type_sbyte}
        }
    }

    pub fn void() -> Self {
        unsafe {
            Type {ptr: jit_type_void}
        }
    }

    pub fn is_void(&self) -> bool {
        unsafe {
            jit_type_get_kind(self.ptr) == (JIT_TYPE_VOID as i32)
        }
    }

    pub fn is_int(&self) -> bool {
        unsafe {
            jit_type_get_kind(self.ptr) == (JIT_TYPE_NINT as i32) ||
            jit_type_get_kind(self.ptr) == (JIT_TYPE_LONG as i32) ||
            jit_type_get_kind(self.ptr) == (JIT_TYPE_INT as i32)
        }
    }

    pub fn is_bool(&self) -> bool {
        unsafe {
            jit_type_get_kind(self.ptr) == (JIT_TYPE_SBYTE as i32)
        }
    }

    pub fn get_pointed_type(&self) -> Type {
        unsafe {
            Type {ptr: jit_type_get_ref(self.ptr)}
        }
    }

    pub fn dump(&self) {
        unsafe {
            printtype(self.ptr)
        }
    }
}