use super::global_context::{ClassDesc, GlobalContext};
use frontend_error::{ok_if_no_error, ErrorAccumulation, FrontendError, FrontendResult};
use model::ast::*;
use std::collections::HashMap;

pub struct FunctionContext<'a> {
    #[allow(dead_code)] // todo remove
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
            self.add_variable_to_env(t, id, &mut env)
                .accumulate_errors_in(&mut errors);
        }
        self.enter_block(&fun.body, env)
            .accumulate_errors_in(&mut errors);

        ok_if_no_error(errors)
    }

    fn enter_block(&self, block: &'a Block, parent_env: VarEnv<'a>) -> FrontendResult<()> {
        // todo
        let mut errors = vec![];
        let mut cur_env = HashMap::new();
        // todo track if block returns

        use self::InnerStmt::*;
        for st in &block.stmts {
            match &st.inner {
                Empty => (),
                Block(bl) => {
                    let mut new_env = parent_env.clone();
                    new_env.extend(&cur_env);
                    self.enter_block(&bl, new_env)
                        .accumulate_errors_in(&mut errors);
                }
                Decl {
                    var_type,
                    var_items,
                } => {
                    self.global_ctx
                        .check_local_var_type(&var_type)
                        .accumulate_errors_in(&mut errors);
                    for (id, init_expr) in var_items {
                        if let Some(_) = cur_env.insert(id.inner.as_ref(), &var_type) {
                            errors.push(FrontendError {
                                err: "Error: variable already defined in this scope".to_string(),
                                span: id.span,
                            })
                        }
                        if let Some(init_expr) = init_expr {
                            // todo should be a complete env; probably write custom, chainable VarEnv
                            self.check_expression_check_type(&init_expr, &var_type, &cur_env)
                                .accumulate_errors_in(&mut errors);
                        }
                    }
                }
                Error => unreachable!(),
                _ => unimplemented!(), // todo remove
            }
        }

        ok_if_no_error(errors)
    }

    fn add_variable_to_env(
        &self,
        var_type: &'a Type,
        name: &'a Ident,
        cur_env: &mut VarEnv<'a>,
    ) -> FrontendResult<()> {
        self.global_ctx.check_local_var_type(var_type)?;
        if let Some(_) = cur_env.insert(name.inner.as_ref(), var_type) {
            Err(vec![FrontendError {
                err: "Error: variable already defined in current scope".to_string(),
                span: name.span,
            }])
        } else {
            Ok(())
        }
    }

    fn check_expression_check_type(
        &self,
        expr: &'a Expr,
        expected_expr_type: &'a Type,
        cur_env: &VarEnv<'a>,
    ) -> FrontendResult<()> {
        let expr_type = self.check_expression_get_type(expr, cur_env)?;
        self.global_ctx
            .check_types_compatibility(expected_expr_type, &expr_type)
    }

    fn check_expression_get_type(
        &self,
        _expr: &'a Expr,
        _cur_env: &VarEnv<'a>,
    ) -> FrontendResult<Type> {
        Err(vec![]) // todo
    }
}
