use super::global_context::{ClassDesc, FunDesc, GlobalContext};
use frontend_error::{ok_if_no_error, ErrorAccumulation, FrontendError, FrontendResult};
use model::ast::*;
use std::collections::HashMap;

pub struct FunctionContext<'a> {
    #[allow(dead_code)] // todo remove
    class_ctx: Option<&'a ClassDesc<'a>>,
    global_ctx: &'a GlobalContext<'a>,
}

enum Env<'a> {
    Root(&'a FunctionContext<'a>),
    Nested {
        #[allow(dead_code)] // todo remove
        parent: &'a Env<'a>,
        locals: HashMap<&'a str, &'a Type>,
    },
}

impl<'a> Env<'a> {
    pub fn new_root(fctx: &'a FunctionContext<'a>) -> Env<'a> {
        Env::Root(fctx)
    }

    pub fn new_nested(parent: &'a Env<'a>) -> Env<'a> {
        Env::Nested {
            parent,
            locals: HashMap::new(),
        }
    }

    pub fn add_variable(&mut self, var_type: &'a Type, name: &'a Ident) -> FrontendResult<()> {
        match self {
            Env::Root(_) => unreachable!(),
            Env::Nested { ref mut locals, .. } => {
                if locals.insert(name.inner.as_ref(), var_type).is_some() {
                    Err(vec![FrontendError {
                        err: "Error: variable already defined in current scope".to_string(),
                        span: name.span,
                    }])
                } else {
                    Ok(())
                }
            }
        }
    }

    pub fn get_variable(&self, _name: &str) -> FrontendResult<InnerType> {
        Err(vec![]) // todo
    }

    pub fn get_function(&self, _name: &str) -> FrontendResult<&'a FunDesc<'a>> {
        Err(vec![]) // todo
    }
}

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
        let root = Env::new_root(&self);
        let mut params_env = Env::new_nested(&root);
        for (t, id) in &fun.args {
            match self.global_ctx.check_local_var_type(&t) {
                Ok(()) => params_env
                    .add_variable(&t, id)
                    .accumulate_errors_in(&mut errors),
                Err(err) => errors.extend(err),
            }
        }

        match (
            self.enter_block(&fun, &fun.body, &params_env),
            &fun.ret_type.inner,
        ) {
            (Ok(true), _) | (Ok(false), InnerType::Void) => (),
            (Ok(false), _) => errors.push(FrontendError {
                err: "Error: detected potential execution path without return".to_string(),
                span: fun.body.span,
            }),
            (Err(err), _) => errors.extend(err),
        }

        ok_if_no_error(errors)
    }

    // return value: if block always returns
    fn enter_block(
        &self,
        fun: &'a FunDef,
        block: &'a Block,
        parent_env: &Env<'a>,
    ) -> FrontendResult<bool> {
        let mut errors = vec![];
        let mut cur_env = Env::new_nested(&parent_env);
        let mut after_ret = false;

        use self::InnerStmt::*;
        for st in &block.stmts {
            if after_ret {
                errors.push(FrontendError {
                    err: "Error: unreachable statement after return statement".to_string(),
                    span: st.span,
                })
            }
            match &st.inner {
                Empty => (),
                Block(bl) => match self.enter_block(fun, &bl, &cur_env) {
                    Ok(does_ret) => after_ret = does_ret,
                    Err(err) => errors.extend(err),
                },
                Decl {
                    var_type,
                    var_items,
                } => {
                    let corr_type = match self.global_ctx.check_local_var_type(&var_type) {
                        Ok(()) => true,
                        Err(err) => {
                            errors.extend(err);
                            false
                        }
                    };
                    for (id, init_expr) in var_items {
                        if corr_type {
                            cur_env
                                .add_variable(&var_type, &id)
                                .accumulate_errors_in(&mut errors);
                        }
                        if let Some(init_expr) = init_expr {
                            self.check_expression_check_type(&init_expr, &var_type.inner, &cur_env)
                                .accumulate_errors_in(&mut errors);
                        }
                    }
                }
                Assign(lhs, rhs) => {
                    // todo (optional) can check both sides of '=' for more errors
                    self.check_if_lvalue(&lhs).accumulate_errors_in(&mut errors);
                    match self.check_expression_get_type(&lhs, &cur_env) {
                        Ok(t) => self
                            .check_expression_check_type(&rhs, &t, &cur_env)
                            .accumulate_errors_in(&mut errors),
                        Err(err) => errors.extend(err),
                    }
                }
                Incr(e) | Decr(e) => {
                    self.check_if_lvalue(&e).accumulate_errors_in(&mut errors);
                    self.check_expression_check_type(&e, &InnerType::Int, &cur_env)
                        .accumulate_errors_in(&mut errors);
                }
                Ret(opt_expr) => {
                    after_ret = true;
                    match opt_expr {
                        Some(ret_expr) => self
                            .check_expression_check_type(&ret_expr, &fun.ret_type.inner, &cur_env)
                            .accumulate_errors_in(&mut errors),
                        None => {
                            if fun.ret_type.inner != InnerType::Void {
                                errors.push(FrontendError {
                                    err: "Error: type of returned expression mismatch declared return type"
                                        .to_string(),
                                    span: st.span,
                                })
                            }
                        }
                    };
                }
                Cond {
                    cond,
                    true_branch,
                    false_branch,
                } => {
                    // todo (optional) check if cond is just true or false
                    self.check_expression_check_type(&cond, &InnerType::Bool, &cur_env)
                        .accumulate_errors_in(&mut errors);
                    let br1_ret = match self.enter_block(fun, &true_branch, &cur_env) {
                        Ok(does_ret) => does_ret,
                        Err(err) => {
                            errors.extend(err);
                            false
                        }
                    };
                    let br2_ret = match false_branch {
                        Some(bl) => match self.enter_block(fun, &bl, &cur_env) {
                            Ok(does_ret) => does_ret,
                            Err(err) => {
                                errors.extend(err);
                                false
                            }
                        },
                        None => true,
                    };
                    after_ret = br1_ret && br2_ret;
                }
                While(cond_expr, body_bl) => {
                    // todo (optional) check if cond is just true or false
                    self.check_expression_check_type(&cond_expr, &InnerType::Bool, &cur_env)
                        .accumulate_errors_in(&mut errors);
                    match self.enter_block(fun, &body_bl, &cur_env) {
                        Ok(does_ret) => after_ret = does_ret,
                        Err(err) => errors.extend(err),
                    }
                }
                Expr(subexpr) => match self.check_expression_get_type(&subexpr, &cur_env) {
                    Ok(_) => (),
                    Err(err) => errors.extend(err),
                },
                Error => unreachable!(),
                _ => errors.push(FrontendError {
                    // todo for each
                    err: "Error: not all statements are supported so far".to_string(),
                    span: st.span,
                }),
            }
        }

        if errors.is_empty() {
            Ok(after_ret)
        } else {
            Err(errors)
        }
    }

    fn check_if_lvalue(&self, expr: &'a Expr) -> FrontendResult<()> {
        use self::InnerExpr::*;
        // todo arrays have only "length" field, and it is read-only!
        match &expr.inner {
            LitVar(_) | ArrayElem { .. } | ObjField { .. } => Ok(()),
            _ => Err(vec![FrontendError{
                err:"Error: required an l-value (options: variable <var>, array elem <expr>[index], or object field <obj>.<field>)".to_string(),
                span:expr.span
            }]),
        }
    }

    fn check_expression_check_type(
        &self,
        expr: &'a Expr,
        expected_expr_type: &'a InnerType,
        cur_env: &Env<'a>,
    ) -> FrontendResult<()> {
        let expr_type = self.check_expression_get_type(expr, cur_env)?;
        // todo, potentially gctx doesn't know span for error
        self.global_ctx
            .check_types_compatibility(expected_expr_type, &expr_type)
    }

    fn check_expression_get_type(
        &self,
        expr: &'a Expr,
        cur_env: &Env<'a>,
    ) -> FrontendResult<InnerType> {
        let mut errors = vec![];

        use self::BinaryOp::*;
        use self::InnerExpr::*;
        use self::InnerType::*;
        use self::InnerUnaryOp::*;
        match &expr.inner {
            LitVar(var) => cur_env.get_variable(&var),
            LitInt(_) => Ok(Int),
            LitBool(_) => Ok(Bool),
            LitStr(_) => Ok(String),
            LitNull => Ok(Null),
            FunCall {
                function_name,
                args,
            } => {
                let fun_desc = cur_env.get_function(function_name.inner.as_ref())?;
                let expected_args_no = fun_desc.args_types.len();
                let got_args_no = args.len();
                if expected_args_no != got_args_no {
                    Err(vec![FrontendError {
                        err: format!(
                            "Error: expected {} argument(s), got {}.",
                            expected_args_no, got_args_no
                        ),
                        span: expr.span,
                    }])
                } else {
                    for (t, a) in fun_desc.args_types.iter().zip(args) {
                        self.check_expression_check_type(&a, &t.inner, &cur_env)
                            .accumulate_errors_in(&mut errors);
                    }

                    if errors.is_empty() {
                        Ok(fun_desc.ret_type.inner.clone())
                    } else {
                        Err(errors)
                    }
                }
            }
            BinaryOp(lhs, op, rhs) => {
                let mut fail_with = |op_str: &str, args: &str| {
                    Err(vec![FrontendError {
                        err: format!(
                            "Error: binary operator '{}' can be applied only to {}",
                            op_str, args
                        ),
                        span: expr.span,
                    }])
                };
                let lhs_res = self.check_expression_get_type(lhs, &cur_env);
                let rhs_res = self.check_expression_get_type(rhs, &cur_env);
                match (lhs_res, rhs_res) {
                    (Ok(lhs_t), Ok(rhs_t)) => match (lhs_t, op, rhs_t) {
                        (Bool, And, Bool) => Ok(Bool),
                        (_, And, _) => fail_with("&&", "boolean expressions"),
                        (Bool, Or, Bool) => Ok(Bool),
                        (_, Or, _) => fail_with("||", "boolean expressions"),
                        (String, Add, String) => Ok(String),
                        (Int, Add, Int) => Ok(Int),
                        (_, Add, _) => fail_with("+", "two integer expressions (sum) or two string expressions (concatenation)"),
                        (Int, Sub, Int) => Ok(Int),
                        (_, Sub, _) => fail_with("-", "integer expressions"),
                        (Int, Mul, Int) => Ok(Int),
                        (_, Mul, _) => fail_with("*", "integer expressions"),
                        (Int, Div, Int) => Ok(Int),
                        (_, Div, _) => fail_with("/", "integer expressions"),
                        (Int, Mod, Int) => Ok(Int),
                        (_, Mod, _) => fail_with("%", "integer expressions"),
                        (Int, LT, Int) => Ok(Bool),
                        (_, LT, _) => fail_with("<", "integer expressions"),
                        (Int, LE, Int) => Ok(Bool),
                        (_, LE, _) => fail_with("<=", "integer expressions"),
                        (Int, GT, Int) => Ok(Bool),
                        (_, GT, _) => fail_with(">", "integer expressions"),
                        (Int, GE, Int) => Ok(Bool),
                        (_, GE, _) => fail_with(">=", "integer expressions"),
                        (Int, EQ, Int) => Ok(Bool),
                        (Bool, EQ, Bool) => Ok(Bool),
                        (String, EQ, String) => Ok(Bool),
                        (_, EQ, _) => fail_with("==", "two operands of same type: integer, boolean or string"),
                        (Int, NE, Int) => Ok(Bool),
                        (Bool, NE, Bool) => Ok(Bool),
                        (String, NE, String) => Ok(Bool),
                        (_, NE, _) => fail_with("!=", "two operands of same type: integer, boolean or string"),
                    },
                    (Ok(_), err @ Err(_)) => err,
                    (err @ Err(_), Ok(_)) => err,
                    (Err(mut err1), Err(err2)) => {
                        err1.extend(err2);
                        Err(err1)
                    }
                }
            }
            UnaryOp(op, e) => {
                let t = self.check_expression_get_type(e, &cur_env)?;
                match (&op.inner, t) {
                    (IntNeg, Int) => Ok(Int),
                    (BoolNeg, Bool) => Ok(Bool),
                    (IntNeg, _) => Err(vec![FrontendError {
                        err: "Error: unary operator '-' can be applied only to integer expressions"
                            .to_string(),
                        span: expr.span,
                    }]),
                    (BoolNeg, _) => Err(vec![FrontendError {
                        err: "Error: unary operator '!' can be applied only to boolean expressions"
                            .to_string(),
                        span: expr.span,
                    }]),
                }
            }
            _ => Err(vec![FrontendError {
                // todo support extensions
                err: "Error: extensions not supported so far".to_string(),
                span: expr.span,
            }]),
        }
    }
}
