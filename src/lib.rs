#[macro_use]
extern crate lalrpop_util;
extern crate colored;

pub mod codemap;
pub mod frontend_error;
pub mod model;
pub mod parser;
pub mod semantics;

// tmp function for development & testing
pub fn compile(filename: &str, code: &str) -> Result<String, String> {
    let codemap = codemap::CodeMap::new(filename, code);
    let res = parser::parse(&codemap);
    let ast = res.map_err(|e| frontend_error::format_errors(&codemap, &e))?;
    let mut sem_anal = semantics::SemanticAnalyzer::new(&ast);
    let res = sem_anal.perform_full_analysis();
    res.map_err(|e| frontend_error::format_errors(&codemap, &e))?;
    Ok(format!("{:?}", ast))
}
