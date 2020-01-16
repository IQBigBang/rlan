use crate::myast::{Node, Op};
use crate::wrapper::{Context, Function, Value, Label, Type};
use std::mem;
use crate::stdlib::*;
use either::Either;
use libc::c_void;

use std::collections::HashMap;

pub struct NativeFunc {
    ptr: *mut c_void,
    argtypes: Vec<Type>,
    ret: Type
}

pub struct Builder {
    pub context: Context,
    pub main: Function,
    pub vtable: HashMap<String, Value>,
    pub ftable: HashMap<String, Either<NativeFunc, Function>>,
}

impl Builder {
    pub fn new() -> Self {
        let context = Context::new();
        let main = context.new_function(&mut [Type::void(); 0], Type::int());
        // initialize built-in functions
        let mut ftable : HashMap<String, Either<NativeFunc, Function>> = HashMap::new();
        ftable.insert(String::from("printint"), 
            Either::Left(NativeFunc {
                ptr: stdlib_printint as *mut c_void,
                argtypes: vec![Type::int()],
                ret: Type::void()
            })
        );
        Builder {context, main, vtable: HashMap::new(), ftable}
    }

    fn get_type(&self, s: &String) -> Type {
        if s == "int" {
            Type::int()
        } else if s == "bool" {
            Type::bool()
        } else if s == "void" {
            Type::void()
        } else {
            panic!("Invalid type")
        }
    }

    pub fn execute(&mut self) -> i32 {
        self.main.compile();
        self.context.finish();
        self.main.standard_execute()
    }

    pub fn visit(&mut self, n: &Node) -> Value {
        match n {
            Node::Number(i) => self.visit_number(i),
            Node::BinOp(lhs, op, rhs) => self.visit_binop(lhs, op, rhs),
            Node::FuncDef(name, args, rettype, body) => self.visit_funcdef(name, args, rettype, body),
            Node::VarDef(name, val) => self.visit_vardef(name, val),
            Node::Ident(name) => self.visit_ident(name),
            Node::Call(name_and_args) => self.visit_call(name_and_args),
            Node::If(cond, then, other) => self.visit_if(cond, then, other),
            Node::Ret(val) => self.visit_ret(val),
            _ => unimplemented!()
        }
    }

    fn visit_number(&mut self, i: &i64) -> Value {
        Value::constant_long(&self.main, *i)
    }

    fn visit_ident(&mut self, name: &String) -> Value {
        let ptr = self.vtable.get(name).expect("Variable doesn't exist.");
        self.main.i_load(ptr)
    }

    fn visit_ret(&mut self, val: &Box<Node>) -> Value {
        let cval = self.visit(&*val);
        self.main.i_return(&cval);
        Value::constant_long(&self.main, 0) // type assured within the call
    }

    fn visit_binop(&mut self, lhs: &Node, op: &Op, rhs: &Node) -> Value {
        let lhs = self.visit(lhs);
        let rhs = self.visit(rhs);
        if lhs.get_type().is_int() && rhs.get_type().is_int() {
            match op {
                Op::Add => self.main.i_add(&lhs, &rhs),
                Op::Sub => self.main.i_sub(&lhs, &rhs),
                Op::Mul => self.main.i_mul(&lhs, &rhs),
                Op::Eql => self.main.i_convert(&self.main.i_eq(&lhs, &rhs), Type::bool()),
                Op::Neq => self.main.i_convert(&self.main.i_ne(&lhs, &rhs), Type::bool()),
                Op::Lwt => self.main.i_convert(&self.main.i_lt(&lhs, &rhs), Type::bool()),
                Op::Lwe => self.main.i_convert(&self.main.i_le(&lhs, &rhs), Type::bool()),
                Op::Grt => self.main.i_convert(&self.main.i_gt(&lhs, &rhs), Type::bool()),
                Op::Gre => self.main.i_convert(&self.main.i_ge(&lhs, &rhs), Type::bool()),
                _ => panic!("Invalid binary operands")
            }
        } else if lhs.get_type().is_bool() && rhs.get_type().is_bool() {
            match op {
                Op::And => self.main.i_and(&lhs, &rhs),
                Op::Or => self.main.i_or(&lhs, &rhs),
                _ => panic!("Invalid binary operands")
            }
        } else {
            print!("Invalid binary operands for operator {:?}", op);
            lhs.get_type().dump(); rhs.get_type().dump();
            println!("");
            panic!("Err")
        }
    }

    fn visit_funcdef(&mut self, name: &String, args: &Vec<(String, String)>, rettype: &String, body: &Vec<Node>) -> Value {
        // get argument types
        let mut argtypes : Vec<Type> = Vec::new();
        for (_, argtype) in args {
            argtypes.push(self.get_type(&argtype));
        }
        // create the function
        let func = self.context.new_function(argtypes.as_mut(), self.get_type(rettype));
        // place it instead of main
        let pre_main = mem::replace(&mut self.main, func);
        // clear the symtable
        self.vtable.clear();
        // load parameters
        let params = self.main.get_params();
        for i in 0..params.len() {
            let param = params[i];
            self.vtable.insert(args[i].0.clone(), param);
        }
        // and save it in case of recursion
        self.ftable.insert(name.clone(),
            Either::Right(self.main), // right = custom function
        );
        // compile body
        for n in body {
            self.visit(n);
        }
        #[cfg(debug_assertions)]
        self.main.dump();
        self.main.compile();
        self.main.dump();
        // place main again
        mem::replace(&mut self.main, pre_main);
        Value::constant_void(&self.main)
    }

    fn visit_call(&mut self, name_and_args: &Vec<Node>) -> Value {
        let fname = match &name_and_args[0] {
            Node::Ident(id) => id,
            _ => panic!("Function name must be an identifier")
        };
        let mut args : Vec<Value> = Vec::new();
        for i in 1..name_and_args.len() {
            args.push(self.visit(name_and_args.get(i).unwrap()));
        };
        let func = match self.ftable.get(fname) {
            None => panic!("Function doesn't exist"),
            Some(f) => f
        };
        match &func {
            Either::Left(nativefunc) => {
                self.main.i_native_call(nativefunc.ptr, args.as_ref(), nativefunc.ret)
            },
            Either::Right(codefunc) => self.main.i_normal_call(codefunc, args.as_ref())
        }
    }

    fn visit_vardef(&mut self, name: &String, val: &Box<Node>) -> Value {
        let val = self.visit(&*val);
        self.vtable.insert(name.to_string(), val);
        Value::constant_void(&self.main) // TODO
    }

    fn visit_if(&mut self, cond: &Box<Node>, then: &Box<Node>, other: &Box<Node>) -> Value {
        let ccond = self.visit(cond);
        Type::bool().is_bool();
        Value::create(&self.main, &Type::bool()).get_type().is_bool();
        self.main.i_convert(&Value::constant_long(&self.main, 0), Type::bool()).get_type().is_bool();
        //println!("{}", Type::bool().is_bool());
        if !ccond.get_type().is_bool() {
            ccond.get_type().dump();
            panic!("Condition type must be a bool");
        } else {
            let elsetree = Label::new();
            self.main.i_branch_if_not(&ccond, &elsetree);
            self.visit(then);
            elsetree.place(&self.main);
            self.visit(other);
            Value::constant_void(&self.main)
        }
    }
}