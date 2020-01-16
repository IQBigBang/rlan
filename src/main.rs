extern crate libc;
extern crate either;
pub mod bindings;
pub mod myast;
pub mod wrapper;
pub mod codegen;
pub mod types;
pub mod stdlib;

#[macro_use] extern crate lalrpop_util;
lalrpop_mod!(pub grammar);

use codegen::Builder;
use wrapper::Value;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Error: the program requires one argument -  the file name");
    }
    let code: String = std::fs::read_to_string(&args[1]).unwrap();
    let parser = grammar::CodeParser::new();
    let parsed = parser.parse(&code).unwrap();
    #[cfg(debug_assertions)]
    println!("{:#?}", parsed);

    let mut builder = Builder::new();
    let mut val = Value::constant_long(&builder.main, 0);
    for n in parsed {
        val = builder.visit(&n);
    };
    builder.main.i_return(&val);
    let res = builder.execute();
    #[cfg(debug_assertions)]
    println!("result = {}", res);
}
