#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(latte);
use self::latte::ProgramParser;

pub mod model;

// todo remove it?
pub fn hello() {
    println!("Hello, world!");
}

// todo clean it
pub fn parse(code: &str) -> model::ast::Program {
    let parser = ProgramParser::new();
    parser.parse(code).unwrap()
}
