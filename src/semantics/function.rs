use std::collections::HashMap;
use super::global_context::{GlobalContext, ClassDesc};
use frontend_error::{FrontendError, FrontendResult, ErrorAccumulation};
use model::ast::*;

#[allow(dead_code)]  // todo remove
pub struct FunctionContext<'a> {
    class_ctx: Option<&'a ClassDesc<'a>>,
    global_ctx: &'a GlobalContext<'a>,
}

// another approach: structure with reference to its parrent environment
type VarEnv<'a> = HashMap<&'a str, &'a Type>;

impl<'a> FunctionContext<'a> {
    pub fn new(cctx: Option<&'a ClassDesc<'a>>, gctx: &'a GlobalContext<'a>) -> Self {
        FunctionContext {
            class_ctx: cctx,
            global_ctx: gctx,
        }
    }

    pub fn analyze_function(&self, fun: &'a FunDef) -> FrontendResult<()> {
        // todo support class context
        let mut errors = vec![];
        let mut env = HashMap::new();
        for (t, id) in &fun.args {
            self.add_variable_to_env(t, id, &mut env).accumulate_errors_in(&mut errors);
        }
        self.entry_block(env, &fun.body).accumulate_errors_in(&mut errors);

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    fn entry_block(&self, _env: VarEnv<'a>, _bl: &'a Block) -> FrontendResult<()> {
        Ok(()) // todo
    }

    fn add_variable_to_env(&self, var_type: &'a Type, name: &'a Ident, cur_env: &mut VarEnv<'a>) -> FrontendResult<()> {
        self.global_ctx.check_local_var_type(var_type)?;
        if let Some(_) = cur_env.insert(name.inner.as_ref(), var_type) {
            Err(vec![FrontendError {
                err: "Error: variable already defined in current scope".to_string(),
                span: name.span,
            }])
        }
        else {
            Ok(())
        }
    }
}
