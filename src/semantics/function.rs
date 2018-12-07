use super::global_context::{GlobalContext, ClassDesc};
use frontend_error::FrontendResult;
use model::ast::*;

#[allow(dead_code)]  // todo remove
pub struct FunctionContext<'a> {
    class_ctx: Option<&'a ClassDesc<'a>>,
    global_ctx: &'a GlobalContext<'a>,
}

impl<'a> FunctionContext<'a> {
    pub fn new(cctx: Option<&'a ClassDesc<'a>>, gctx: &'a GlobalContext<'a>) -> Self {
        FunctionContext {
            class_ctx: cctx,
            global_ctx: gctx,
        }
    }

    pub fn analyze_function(&self, _fun: &'a FunDef) -> FrontendResult<()> {
        Ok(()) // todo
    }
}
