use crate::myast::{Node, Op};
use crate::wrapper::{Context, Function, Value, Label, Type, TpIndex, TPINDEX_BOOL, TPINDEX_INT64, TPINDEX_VOID_OR_UNKNOWN};
use std::mem;
use crate::stdlib::*;
use either::Either;

use std::collections::HashMap;

pub struct TypeDescriptor {
    pub index: TpIndex,
    pub name: String,
    pub tp: Type
}

pub struct Builder {
    pub context: Context,
    pub main: Function,
    pub vtable: HashMap<String, Value>,
    pub ftable: HashMap<String, (Either<NativeFunc, Function>, Vec<TpIndex>, TpIndex)>, // function, arg types, ret type
    pub types: Vec<TypeDescriptor>
}

impl Builder {
    pub fn new() -> Self {
        let context = Context::new();
        let main = context.new_function(&mut [Type::void(); 0], Type::int());
        // initialize built-in types
        let types = vec![
            TypeDescriptor {index: TPINDEX_VOID_OR_UNKNOWN, name: String::new(), tp: Type::void()}, // void = 0
            TypeDescriptor {index: TPINDEX_INT64, name: String::from("int"), tp: Type::long()}, // int = 1
            TypeDescriptor {index: TPINDEX_BOOL, name: String::from("bool"), tp: Type::long()}, // bool = 2
        ];
        // initialize built-in functions
        let mut ftable : HashMap<String, (Either<NativeFunc, Function>, Vec<TpIndex>, TpIndex)> = HashMap::new();
        ftable.insert(String::from("printint"), (
            Either::Left(stdlib_printint as NativeFunc),
            vec![TPINDEX_INT64],
            TPINDEX_VOID_OR_UNKNOWN
        ));
        Builder {context, main, vtable: HashMap::new(), ftable, types}
    }

    fn get_type_descriptor(&self, s: &String) -> &TypeDescriptor {
        let mut res = None;
        for td in &self.types {
            if &td.name == s {
                res = Some(td)
            }
        };
        &res.expect("Invalid type")
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
        *self.vtable.get(name).expect("Variable doesn't exist.")
    }

    fn visit_ret(&mut self, val: &Box<Node>) -> Value {
        let cval = self.visit(&*val);
        self.main.i_return(&cval);
        Value::constant_long(&self.main, 0) // type assured within the call
    }

    fn visit_binop(&mut self, lhs: &Node, op: &Op, rhs: &Node) -> Value {
        let lhs = self.visit(lhs);
        let rhs = self.visit(rhs);
        if lhs.type_index() == TPINDEX_INT64 && rhs.type_index() == TPINDEX_INT64 {
            match op {
                Op::Add => self.main.i_add(&lhs, &rhs).with_type(TPINDEX_INT64),
                Op::Sub => self.main.i_sub(&lhs, &rhs).with_type(TPINDEX_INT64),
                Op::Mul => self.main.i_mul(&lhs, &rhs).with_type(TPINDEX_INT64),
                Op::Eql => self.main.i_eq(&lhs, &rhs).with_type(TPINDEX_BOOL),
                Op::Neq => self.main.i_ne(&lhs, &rhs).with_type(TPINDEX_BOOL),
                Op::Lwt => self.main.i_lt(&lhs, &rhs).with_type(TPINDEX_BOOL),
                Op::Lwe => self.main.i_le(&lhs, &rhs).with_type(TPINDEX_BOOL),
                Op::Grt => self.main.i_gt(&lhs, &rhs).with_type(TPINDEX_BOOL),
                Op::Gre => self.main.i_ge(&lhs, &rhs).with_type(TPINDEX_BOOL),
                _ => panic!("Invalid binary operands")
            }
        } else if lhs.type_index() == TPINDEX_BOOL && rhs.type_index() == TPINDEX_BOOL {
            match op {
                Op::And => self.main.i_and(&lhs, &rhs).with_type(TPINDEX_BOOL),
                Op::Or => self.main.i_or(&lhs, &rhs).with_type(TPINDEX_BOOL),
                _ => panic!("Invalid binary operands")
            }
        } else {
            panic!(format!("Invalid binary operands {} and {}", lhs.type_index(), rhs.type_index()))
        }
    }

    fn visit_funcdef(&mut self, name: &String, args: &Vec<(String, String)>, rettype: &String, body: &Vec<Node>) -> Value {
        // get argument types
        let mut argtypes : Vec<Type> = Vec::new();
        let mut argtypesindexes : Vec<i32> = Vec::new();
        for (_, argtype) in args {
            let typedesc = self.get_type_descriptor(&argtype);
            argtypesindexes.push(typedesc.index);
            argtypes.push(typedesc.tp);
        }
        // create the function
        let func = self.context.new_function(argtypes.as_mut(), self.get_type_descriptor(rettype).tp);
        // place it instead of main
        let pre_main = mem::replace(&mut self.main, func);
        // clear the symtable
        self.vtable.clear();
        // load parameters
        let params = self.main.get_params();
        for i in 0..params.len() {
            self.vtable.insert(args[i].0.clone(), params[i].with_type(argtypesindexes[i]));
        }
        // and save it in case of recursion
        self.ftable.insert(name.clone(), (
            Either::Right(self.main), // right = custom function 
            argtypesindexes, 
            self.get_type_descriptor(rettype).index
        ));
        // compile body
        for n in body {
            self.visit(n);
        }
        #[cfg(Debug)]
        self.main.dump();
        self.main.compile();
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
        for i in 0..args.len() {
            if args[i].type_index() != func.1[i] {
                panic!("Invalid argument type");
            }
        }
        match &func.0 {
            Either::Left(nativefunc) => {
                let raw_type : Type = self.types[func.2 as usize].tp;
                self.main.i_native_call(*nativefunc, args.as_ref(), raw_type).with_type(func.2)
            },
            Either::Right(codefunc) => self.main.i_normal_call(codefunc, args.as_ref()).with_type(func.2)
        }
    }

    fn visit_if(&mut self, cond: &Box<Node>, then: &Box<Node>, other: &Box<Node>) -> Value {
        let ccond = self.visit(cond);
        if ccond.type_index() != TPINDEX_BOOL {
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