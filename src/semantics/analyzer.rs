use super::function::FunctionContext;
use super::global_context::GlobalContext;
use frontend_error::{ok_if_no_error, ErrorAccumulation, FrontendError, FrontendResult};
use model::ast::*;

pub struct SemanticAnalyzer<'a> {
    ast: &'a Program,
    ctx: Option<GlobalContext<'a>>,
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(prog: &'a Program) -> Self {
        SemanticAnalyzer {
            ast: prog,
            ctx: None,
        }
    }

    pub fn perform_full_analysis(&mut self) -> FrontendResult<()> {
        self.calculate_global_context()?;
        self.analyze_functions()?;
        self.check_main_signature()
    }

    pub fn get_global_ctx(&self) -> Option<&GlobalContext<'a>> {
        self.ctx.as_ref()
    }

    fn calculate_global_context(&mut self) -> FrontendResult<()> {
        if self.ctx.is_some() {
            return Ok(());
        }

        match GlobalContext::from(self.ast) {
            Ok(ctx) => {
                self.ctx = Some(ctx);
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn analyze_functions(&mut self) -> FrontendResult<()> {
        let mut errors = vec![];
        let err_msg = "Global analysis succeeded before function body analysis";
        let gctx = self.ctx.as_ref().expect(err_msg);
        let gfun_ctx = FunctionContext::new(None, &gctx);
        for def in &self.ast.defs {
            match def {
                TopDef::FunDef(fun) => {
                    gfun_ctx
                        .analyze_function(&fun)
                        .accumulate_errors_in(&mut errors);
                }
                TopDef::ClassDef(cl) => {
                    let cl_desc = gctx.get_class_description(&cl.name.inner).expect(err_msg);
                    let cl_ctx = FunctionContext::new(Some(cl_desc), &gctx);
                    for it in &cl.items {
                        match &it.inner {
                            InnerClassItemDef::Field(_, _) => (),
                            InnerClassItemDef::Method(fun) => {
                                cl_ctx
                                    .analyze_function(&fun)
                                    .accumulate_errors_in(&mut errors);
                            }
                            InnerClassItemDef::Error => unreachable!(),
                        }
                    }
                }
                TopDef::Error => unreachable!(),
            }
        }

        ok_if_no_error(errors)
    }

    fn check_main_signature(&mut self) -> FrontendResult<()> {
        let err_msg = "Global analysis succeeded before function body analysis";
        let gctx = self.ctx.as_ref().expect(err_msg);
        match gctx.get_function_description("main") {
            Some(f) => {
                if f.ret_type.inner == InnerType::Int && f.args_types.is_empty() {
                    Ok(())
                } else {
                    Err(vec![FrontendError {
                    err: "Error: main function has invalid signature, it must return int and take no arguments".to_string(),
                    span: EMPTY_SPAN, // we could have correct span here, though
                }])
                }
            }
            None => Err(vec![FrontendError {
                err: "Error: main function not found".to_string(),
                span: EMPTY_SPAN,
            }]),
        }
    }
}
