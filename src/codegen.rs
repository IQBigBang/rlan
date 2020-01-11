use crate::myast::{Node, Op};
use crate::wrapper::{Context, Function, Value, Label, Type};
use std::mem;

use std::collections::HashMap;

pub struct Builder {
    pub context: Context,
    pub main: Function,
    pub vtable: HashMap<String, Value>,
    pub ftable: HashMap<String, Function>,
}

fn get_type(name: &str) -> Type {
    if name == "int" {
        Type::long()
    } else if name == "bool" {
        Type::sbyte()
    } else {
        panic!("Unknown type")
    }
}

fn get_type_string(name: &String) -> Type {
    get_type(name.as_ref())
}

impl Builder {
    pub fn new() -> Self {
        let context = Context::new();
        let main = context.new_function(&mut [Type::void(); 0], Type::int());
        Builder {context, main, vtable: HashMap::new(), ftable: HashMap::new()}
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
        Value::constant_long(&self.main, 0)
    }

    fn visit_binop(&mut self, lhs: &Node, op: &Op, rhs: &Node) -> Value {
        let lhs = self.visit(lhs);
        let rhs = self.visit(rhs);
        if lhs.get_type() == get_type("int") && rhs.get_type() == get_type("int") {
            match op {
                Op::Add => self.main.i_add(&lhs, &rhs),
                Op::Sub => self.main.i_sub(&lhs, &rhs),
                Op::Mul => self.main.i_mul(&lhs, &rhs),
                Op::Eql => self.main.i_convert(&self.main.i_eq(&lhs, &rhs), get_type("bool")),
                _ => unimplemented!()
            }
        } else if lhs.get_type() == get_type("bool") && rhs.get_type() == get_type("bool") {
            panic!("Invalid binary operands") // bool type
        } else {
            panic!("Invalid binary operands")
        }
    }

    fn visit_funcdef(&mut self, name: &String, args: &Vec<(String, String)>, rettype: &String, body: &Vec<Node>) -> Value {
        // get argument types
        let mut argtypes : Vec<Type> = Vec::new();
        for (_, argtype) in args {
            argtypes.push(get_type_string(argtype));
        }
        // create the function
        let func = self.context.new_function(argtypes.as_mut(), get_type_string(rettype));
        // and save it in case of recursion
        self.ftable.insert(name.clone(), func);
        // place it instead of main
        let pre_main = mem::replace(&mut self.main, func);
        // and clear symtable
        self.vtable.clear();
        // load parameters
        let params = self.main.get_params();
        for i in 0..params.len() {
            self.vtable.insert(args[i].0.clone(), params[i]);
        }
        // compile body
        for n in body {
            self.visit(n);
        }
        self.main.optimize();
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
        self.main.i_normal_call(func, args.as_ref())
    }

    fn visit_if(&mut self, cond: &Box<Node>, then: &Box<Node>, other: &Box<Node>) -> Value {
        let ccond = self.visit(cond);
        if ccond.get_type() != get_type("bool") {
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