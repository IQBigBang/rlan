extern crate libc;
pub mod bindings;
pub mod myast;
pub mod wrapper;
pub mod codegen;
pub mod types;

#[macro_use] extern crate lalrpop_util;
lalrpop_mod!(pub grammar);

use codegen::Builder;
use wrapper::Value;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let code: String = std::fs::read_to_string(&args[1]).unwrap();
    let parser = grammar::CodeParser::new();
    let parsed = parser.parse(&code).unwrap();
    println!("{:#?}", parsed);

    let mut builder = Builder::new();
    let mut val = Value::constant_long(&builder.main, 0);
    for n in parsed {
        val = builder.visit(&n);
    };
    builder.main.i_return(&val);

    let res = builder.execute();

    println!("result = {}", res);
}
