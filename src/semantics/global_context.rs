use std::collections::HashMap;
use model::ast::*;
use frontend_error::{FrontendError, FrontendResult};

pub struct GlobalContext<'a> {
    #[allow(dead_code)] // todo remove
    classes: HashMap<&'a str, ClassDesc<'a>>,
    functions: HashMap<&'a str, FunDesc<'a>>,
}

#[allow(dead_code)] // todo remove
struct ClassDesc<'a> {  // todo
    name: &'a str,
}

struct FunDesc<'a> {
    pub ret_type: &'a Type,
    pub name: &'a str,
    pub args_types: Vec<&'a Type>,
}

impl<'a> GlobalContext<'a> {
    fn new_with_builtins() -> Self {
        // todo add builtins
        GlobalContext {
            classes: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn from(prog: &'a Program) -> FrontendResult<Self> {
        let mut result = GlobalContext::new_with_builtins();
        let mut errors = vec![];
        for def in &prog.defs {
            match def {
                TopDef::FunDef(fun) => {
                    let fun_desc = FunDesc::from(&fun);
                    if result.functions.contains_key(fun_desc.name) {
                        errors.push(FrontendError {
                            err: "Error: function redefinition".to_string(),
                            span: fun.name.span,
                         });
                    }
                    else {
                        result.functions.insert(fun_desc.name, fun_desc);
                    }
                },
                TopDef::ClassDef(_cl) => {
                    unimplemented!() // todo
                },
                TopDef::Error => unreachable!(),
            }
        }

        // todo step 2:
        // check if types in classes and functions are defined (and are not void, void[], etc)

        if errors.is_empty() { Ok(result) } else { Err(errors) }
    }
}

impl<'a> ClassDesc<'a> {
    #[allow(dead_code)] // todo remove
    pub fn from(_cldef: &'a ClassDef) -> FrontendResult<Self> {
        unimplemented!() // todo
    }
}

impl<'a> FunDesc<'a> {
    pub fn from(fundef: &'a FunDef) -> Self {
        FunDesc {
            ret_type: &fundef.ret_type,
            name: &fundef.name.inner,
            args_types: fundef.args.iter().map(|(t, _)| t).collect(),
        }
    }
}
