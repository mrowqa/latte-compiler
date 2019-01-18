use super::global_context::{ClassDesc, FunDesc, GlobalContext, TypeWrapper};
use frontend_error::{ok_if_no_error, ErrorAccumulation, FrontendError, FrontendResult};
use model::ast::*;
use std::collections::HashMap;

pub struct FunctionContext<'a> {
    class_ctx: Option<&'a ClassDesc>,
    global_ctx: &'a GlobalContext,
}

enum Env<'a> {
    Root(&'a FunctionContext<'a>),
    Nested {
        parent: &'a Env<'a>,
        locals: HashMap<String, Type>,
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

    pub fn add_variable(&mut self, var_type: Type, name: Ident) -> FrontendResult<()> {
        if name.inner == THIS_VAR {
            return Err(vec![FrontendError {
                err: "Error: \"this\" variable is reserved for class methods and can't be defined"
                    .to_string(),
                span: name.span,
            }]);
        }
        match self {
            Env::Root(_) => unreachable!(),
            Env::Nested { ref mut locals, .. } => {
                if locals.insert(name.inner, var_type).is_some() {
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

    // returns type & is member of a class
    pub fn get_variable(&self, name: &str, span: Span) -> FrontendResult<(InnerType, bool)> {
        match self {
            Env::Root(ctx) => {
                let mut err_msg = None;
                if let Some(cctx) = ctx.class_ctx {
                    if name == THIS_VAR {
                        return Ok((InnerType::Class(cctx.get_name().to_string()), false));
                    }
                    match cctx.get_item(ctx.global_ctx, name) {
                        Some(TypeWrapper::Var(t)) => return Ok((t.inner.clone(), true)),
                        Some(TypeWrapper::Fun(_)) => {
                            err_msg = Some("Error: expected variable, found a class method")
                        }
                        None => (),
                    }
                }
                let err_msg = match err_msg {
                    Some(e) => e,
                    None => match ctx.global_ctx.get_function_description(name) {
                        Some(_) => "Error: expected variable, found a function",
                        None => "Error: variable not defined",
                    },
                };
                Err(vec![FrontendError {
                    err: err_msg.to_string(),
                    span,
                }])
            }
            Env::Nested { locals, parent } => match locals.get(name) {
                Some(t) => Ok((t.inner.clone(), false)),
                None => parent.get_variable(name, span),
            },
        }
    }

    // returns fun desc & is a class method
    pub fn get_function(&self, name: &str, span: Span) -> FrontendResult<(&'a FunDesc, bool)> {
        match self {
            Env::Root(ctx) => {
                let mut err_msg = None;
                if let Some(cctx) = ctx.class_ctx {
                    match cctx.get_item(ctx.global_ctx, name) {
                        Some(TypeWrapper::Fun(f)) => return Ok((f, true)),
                        Some(TypeWrapper::Var(_)) => {
                            err_msg = Some("Error: expected function, found a class field")
                        }
                        None => (),
                    }
                }
                let err_msg = match err_msg {
                    Some(e) => e,
                    None => match ctx.global_ctx.get_function_description(name) {
                        Some(f) => return Ok((f, false)),
                        None => "Error: function not defined",
                    },
                };
                Err(vec![FrontendError {
                    err: err_msg.to_string(),
                    span,
                }])
            }
            Env::Nested { locals, parent } => match locals.get(name) {
                Some(_) => Err(vec![FrontendError {
                    err: "Error: expected function, got a variable".to_string(),
                    span,
                }]),
                None => parent.get_function(name, span),
            },
        }
    }
}

impl<'a> FunctionContext<'a> {
    pub fn new(cctx: Option<&'a ClassDesc>, gctx: &'a GlobalContext) -> Self {
        FunctionContext {
            class_ctx: cctx,
            global_ctx: gctx,
        }
    }

    pub fn analyze_function(&self, fun: &'a mut FunDef) -> FrontendResult<()> {
        let mut errors = vec![];
        let root = Env::new_root(&self);
        let mut params_env = Env::new_nested(&root);
        for (t, id) in &fun.args {
            match self.global_ctx.check_local_var_type(&t) {
                Ok(()) => params_env
                    .add_variable(t.clone(), id.clone())
                    .accumulate_errors_in(&mut errors),
                Err(err) => errors.extend(err),
            }
        }

        match (
            self.enter_block(&fun.ret_type, &mut fun.body, &params_env),
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
        ret_type: &'a Type,
        block: &'a mut Block,
        parent_env: &Env<'a>,
    ) -> FrontendResult<bool> {
        let mut errors = vec![];
        let mut cur_env = Env::new_nested(&parent_env);
        let mut after_ret = false;

        use self::InnerStmt::*;
        for st in &mut block.stmts {
            // it could be a warning, though
            // (we need to accept unreachable code)
            // if after_ret {
            //     errors.push(FrontendError {
            //         err: "Error: unreachable statement after return statement".to_string(),
            //         span: st.span,
            //     })
            // }
            let st_span = st.span; // making borrow checker happy
            match &mut st.inner {
                Empty => (),
                Block(ref mut bl) => match self.enter_block(ret_type, bl, &cur_env) {
                    Ok(does_ret) => after_ret |= does_ret,
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
                                .add_variable(var_type.clone(), id.clone())
                                .accumulate_errors_in(&mut errors);
                        }
                        if let Some(ref mut init_expr) = init_expr {
                            self.check_expression_check_type(init_expr, &var_type.inner, &cur_env)
                                .accumulate_errors_in(&mut errors);
                        }
                    }
                }
                Assign(ref mut lhs, ref mut rhs) => {
                    // todo (optional) can check both sides of '=' for more errors
                    match self.check_expression_get_type(lhs, &cur_env) {
                        Ok(t) => {
                            self.check_if_lvalue(&lhs).accumulate_errors_in(&mut errors);
                            self.check_expression_check_type(rhs, &t, &cur_env)
                                .accumulate_errors_in(&mut errors);
                        }
                        Err(err) => errors.extend(err),
                    }
                }
                Incr(ref mut e) | Decr(ref mut e) => {
                    self.check_expression_check_type(e, &InnerType::Int, &cur_env)
                        .accumulate_errors_in(&mut errors);
                    self.check_if_lvalue(&e).accumulate_errors_in(&mut errors);
                }
                Ret(opt_expr) => {
                    after_ret = true;
                    match opt_expr {
                        Some(ref mut ret_expr) => self
                            .check_expression_check_type(ret_expr, &ret_type.inner, &cur_env)
                            .accumulate_errors_in(&mut errors),
                        None => {
                            if ret_type.inner != InnerType::Void {
                                errors.push(FrontendError {
                                    err: "Error: type of returned expression mismatch declared return type"
                                        .to_string(),
                                    span: st_span,
                                })
                            }
                        }
                    };
                }
                Cond {
                    ref mut cond,
                    ref mut true_branch,
                    false_branch,
                } => {
                    self.check_expression_check_type(cond, &InnerType::Bool, &cur_env)
                        .accumulate_errors_in(&mut errors);
                    let cond_state = match &cond.inner {
                        InnerExpr::LitBool(cond_val) => Some(cond_val),
                        _ => None,
                    };
                    let br1_ret = match self.enter_block(ret_type, true_branch, &cur_env) {
                        Ok(does_ret) => does_ret,
                        Err(err) => {
                            errors.extend(err);
                            false
                        }
                    };
                    let br2_ret = match false_branch {
                        Some(ref mut bl) => match self.enter_block(ret_type, bl, &cur_env) {
                            Ok(does_ret) => does_ret,
                            Err(err) => {
                                errors.extend(err);
                                false
                            }
                        },
                        None => false,
                    };
                    after_ret |= match cond_state {
                        Some(true) => br1_ret,
                        Some(false) => br2_ret,
                        None => br1_ret && br2_ret,
                    };
                }
                While(ref mut cond_expr, ref mut body_bl) => {
                    self.check_expression_check_type(cond_expr, &InnerType::Bool, &cur_env)
                        .accumulate_errors_in(&mut errors);
                    match self.enter_block(ret_type, body_bl, &cur_env) {
                        Ok(does_ret) => after_ret |= does_ret,
                        Err(err) => errors.extend(err),
                    };
                    if let InnerExpr::LitBool(ret) = &cond_expr.inner {
                        // while (true) just loops, so we don't have to check if we return after it
                        // while (false) just need to be skipped,
                        after_ret |= *ret;
                    };
                }
                ForEach {
                    iter_type,
                    iter_name,
                    ref mut array,
                    body,
                } => {
                    let mut new_env = Env::new_nested(&cur_env);
                    match self.global_ctx.check_local_var_type(&iter_type) {
                        Ok(()) => {
                            new_env
                                .add_variable(iter_type.clone(), iter_name.clone())
                                .accumulate_errors_in(&mut errors);

                            self.check_expression_check_type(
                                array,
                                &InnerType::Array(Box::new(iter_type.inner.clone())),
                                &cur_env,
                            )
                            .accumulate_errors_in(&mut errors)
                        }
                        Err(err) => errors.extend(err),
                    }

                    match self.enter_block(ret_type, body, &new_env) {
                        Ok(does_ret) => after_ret |= does_ret,
                        Err(err) => errors.extend(err),
                    }
                }
                Expr(ref mut subexpr) => match self.check_expression_get_type(subexpr, &cur_env) {
                    Ok(_) => (),
                    Err(err) => errors.extend(err),
                },
                Error => unreachable!(),
            }
        }

        if errors.is_empty() {
            Ok(after_ret)
        } else {
            Err(errors)
        }
    }

    // requirement: check_expr called on expr beforehand
    fn check_if_lvalue(&self, expr: &'a Expr) -> FrontendResult<()> {
        use self::InnerExpr::*;
        match &expr.inner {
            LitVar(_) | ArrayElem { .. } => Ok(()),
            ObjField { is_obj_an_array, .. } => match is_obj_an_array {
                Some(true) => Err(vec![FrontendError {
                    err: "Error: only class objects have mutable fields".to_string(),
                    span: expr.span
                }]),
                Some(false) => Ok(()), // it's a class
                None => unreachable!(), // this function requires analysis to be done beforehand
            },
            _ => Err(vec![FrontendError {
                err: "Error: required an l-value (options: variable <var>, array elem <expr>.[index], or object field <obj>.<field>)".to_string(),
                span: expr.span,
            }]),
        }
    }

    fn check_expression_check_type(
        &self,
        expr: &'a mut Expr,
        expected_expr_type: &'a InnerType,
        cur_env: &Env<'a>,
    ) -> FrontendResult<()> {
        let expr_type = self.check_expression_get_type(expr, cur_env)?;
        self.global_ctx
            .check_types_compatibility(expected_expr_type, &expr_type, expr.span)?;
        if *expected_expr_type != expr_type {
            expr.inner = InnerExpr::CastType(
                Box::new(ItemWithSpan {
                    inner: expr.inner.clone(), // clone to satisfy borrow checker, usually should be small expr, anyway
                    span: expr.span,
                }),
                expected_expr_type.clone(),
            );
        }
        Ok(())
    }

    fn check_expression_get_type(
        &self,
        expr: &'a mut Expr,
        cur_env: &Env<'a>,
    ) -> FrontendResult<InnerType> {
        let expr_span = expr.span; // making borrow checker happy
        let front_err = |err| {
            Err(vec![FrontendError {
                err,
                span: expr_span,
            }])
        };

        let validate_fun_call = |fun_desc: &FunDesc, args: &mut Vec<Box<Expr>>| {
            let mut errors = vec![];
            let expected_args_no = fun_desc.args_types.len();
            let got_args_no = args.len();
            if expected_args_no != got_args_no {
                front_err(format!(
                    "Error: expected {} argument(s), got {}.",
                    expected_args_no, got_args_no
                ))
            } else {
                for (t, ref mut a) in fun_desc.args_types.iter().zip(args) {
                    self.check_expression_check_type(a, &t.inner, &cur_env)
                        .accumulate_errors_in(&mut errors);
                }

                if errors.is_empty() {
                    Ok(fun_desc.ret_type.inner.clone())
                } else {
                    Err(errors)
                }
            }
        };

        let mut override_expr = None;
        use self::BinaryOp::*;
        use self::InnerExpr::*;
        use self::InnerType::*;
        use self::InnerUnaryOp::*;
        let result = match &mut expr.inner {
            LitVar(var) => match cur_env.get_variable(&var, expr.span) {
                Ok((var_type, true)) => {
                    override_expr = Some(InnerExpr::ObjField {
                        obj: Box::new(ItemWithSpan {
                            span: expr.span,
                            inner: InnerExpr::LitVar(THIS_VAR.to_string()),
                        }),
                        is_obj_an_array: Some(false),
                        field: ItemWithSpan {
                            span: expr.span,
                            inner: var.to_string(),
                        },
                    });
                    Ok(var_type)
                }
                Ok((var_type, false)) => Ok(var_type),
                Err(err) => Err(err),
            },
            LitInt(_) => Ok(Int),
            LitBool(_) => Ok(Bool),
            LitStr(_) => Ok(String),
            LitNull => Ok(Null),
            CastType(_, _) => unreachable!(), // we add it after processing some node (it is implicit cast)
            FunCall {
                function_name,
                ref mut args,
            } => match cur_env.get_function(function_name.inner.as_ref(), function_name.span) {
                Ok((fun_desc, is_class_member)) => {
                    let result = validate_fun_call(&fun_desc, args);
                    if is_class_member {
                        override_expr = Some(InnerExpr::ObjMethodCall {
                            obj: Box::new(ItemWithSpan {
                                span: function_name.span,
                                inner: InnerExpr::LitVar(THIS_VAR.to_string()),
                            }),
                            method_name: function_name.clone(),
                            args: args.to_vec(), // copy to satisfy borrow checker, usually should be small objects
                        });
                    }
                    result
                }
                Err(err) => Err(err),
            },
            BinaryOp(ref mut lhs, op, ref mut rhs) => {
                let fail_with = |op_str: &str, args: &str| {
                    front_err(format!(
                        "Error: binary operator '{}' can be applied only to {}",
                        op_str, args
                    ))
                };
                let lhs_res = self.check_expression_get_type(lhs, &cur_env);
                let rhs_res = self.check_expression_get_type(rhs, &cur_env);
                match (lhs_res, rhs_res) {
                    (Ok(lhs_t), Ok(rhs_t)) => match (lhs_t, op, rhs_t) {
                        (Bool, And, Bool) | (Bool, Or, Bool) => Ok(Bool),
                        (_, And, _) => fail_with("&&", "boolean expressions"),
                        (_, Or, _) => fail_with("||", "boolean expressions"),
                        (String, Add, String) => Ok(String),
                        (Int, Add, Int) | (Int, Sub, Int)
                        | (Int, Mul, Int) | (Int, Div, Int) | (Int, Mod, Int) => Ok(Int),
                        (_, Add, _) => fail_with("+", "two integer expressions (sum) or two string expressions (concatenation)"),
                        (_, Sub, _) => fail_with("-", "integer expressions"),
                        (_, Mul, _) => fail_with("*", "integer expressions"),
                        (_, Div, _) => fail_with("/", "integer expressions"),
                        (_, Mod, _) => fail_with("%", "integer expressions"),
                        (Int, LT, Int) | (Int, LE, Int)
                        | (Int, GT, Int) | (Int, GE, Int)
                        | (Int, EQ, Int) | (Int, NE, Int) => Ok(Bool),
                        (_, LT, _) => fail_with("<", "integer expressions"),
                        (_, LE, _) => fail_with("<=", "integer expressions"),
                        (_, GT, _) => fail_with(">", "integer expressions"),
                        (_, GE, _) => fail_with(">=", "integer expressions"),
                        (Bool, EQ, Bool) | (String, EQ, String) => Ok(Bool),
                        (Class(_), EQ, Null) | (Null, EQ, Class(_))
                        | (Array(_), EQ, Null) | (Null, EQ, Array(_)) => Ok(Bool),
                        (_, EQ, _) => fail_with("==", "two operands of same type: integer, boolean and string, or used to check if array or class reference is null"),
                        (Bool, NE, Bool) | (String, NE, String) => Ok(Bool),
                        (Class(_), NE, Null) | (Null, NE, Class(_))
                        | (Array(_), NE, Null) | (Null, NE, Array(_)) => Ok(Bool),
                        (_, NE, _) => fail_with("!=", "two operands of same type: integer, boolean and string, or used to check if array or class reference is null"),
                    },
                    (Ok(_), err @ Err(_)) => err,
                    (err @ Err(_), Ok(_)) => err,
                    (Err(mut err1), Err(err2)) => {
                        err1.extend(err2);
                        Err(err1)
                    }
                }
            }
            UnaryOp(op, ref mut e) => {
                let t = self.check_expression_get_type(e, &cur_env)?;
                match (&op.inner, t) {
                    (IntNeg, Int) => Ok(Int),
                    (BoolNeg, Bool) => Ok(Bool),
                    (IntNeg, _) => front_err(
                        "Error: unary operator '-' can be applied only to integer expressions"
                            .to_string(),
                    ),
                    (BoolNeg, _) => front_err(
                        "Error: unary operator '!' can be applied only to boolean expressions"
                            .to_string(),
                    ),
                }
            }
            NewArray {
                elem_type,
                ref mut elem_cnt,
            } => {
                let type_ok = self.global_ctx.check_local_var_type(&elem_type);
                let cnt_ok = self.check_expression_check_type(elem_cnt, &Int, &cur_env);
                match (type_ok, cnt_ok) {
                    (Ok(()), Ok(())) => Ok(Array(Box::new(elem_type.inner.clone()))),
                    (Ok(_), Err(err)) => Err(err),
                    (Err(err), Ok(_)) => Err(err),
                    (Err(mut err1), Err(err2)) => {
                        err1.extend(err2);
                        Err(err1)
                    }
                }
            }
            ArrayElem {
                ref mut array,
                ref mut index,
            } => {
                let mut errors = vec![];
                self.check_expression_check_type(index, &Int, &cur_env)
                    .accumulate_errors_in(&mut errors);
                let res = match self.check_expression_get_type(array, &cur_env) {
                    Ok(Array(t)) => Some(t),
                    Ok(_) => {
                        errors.push(FrontendError {
                            err: "Error: only arrays can be indexed".to_string(),
                            span: expr.span,
                        });
                        None
                    }
                    Err(err) => {
                        errors.extend(err);
                        None
                    }
                };
                if let (Some(t), true) = (res, errors.is_empty()) {
                    Ok(*t)
                } else {
                    Err(errors)
                }
            }
            NewObject(obj_type) => {
                self.global_ctx.check_local_var_type(&obj_type)?;
                if let Class(_) = obj_type.inner {
                    Ok(obj_type.inner.clone())
                } else {
                    front_err("Error: you can use new only with class and array types".to_string())
                }
            }
            ObjField {
                ref mut obj,
                ref mut is_obj_an_array,
                field,
            } => match self.check_expression_get_type(obj, &cur_env) {
                Ok(Class(cl_name)) => {
                    *is_obj_an_array = Some(false);
                    let desc = self
                        .global_ctx
                        .get_class_description(&cl_name)
                        .expect("check_expression_get_type returns correct types");
                    match desc.get_item(self.global_ctx, &field.inner) {
                        Some(TypeWrapper::Var(t)) => Ok(t.inner.clone()),
                        Some(TypeWrapper::Fun(_)) => {
                            front_err(format!("Error: {} is a method, not a field", field.inner))
                        }
                        None => front_err(format!(
                            "Error: {} is not defined for class {}",
                            field.inner, cl_name
                        )),
                    }
                }
                Ok(Array(_)) => {
                    *is_obj_an_array = Some(true);
                    if field.inner == "length" {
                        Ok(Int)
                    } else {
                        front_err("Error: array's only field is length".to_string())
                    }
                }
                Ok(_) => front_err("Error: only classes and arrays have fields".to_string()),
                Err(err) => Err(err),
            },
            ObjMethodCall {
                ref mut obj,
                method_name,
                ref mut args,
            } => match self.check_expression_get_type(obj, &cur_env) {
                Ok(Class(cl_name)) => {
                    let desc = self
                        .global_ctx
                        .get_class_description(&cl_name)
                        .expect("check_expression_get_type returns correct types");
                    match desc.get_item(self.global_ctx, &method_name.inner) {
                        Some(TypeWrapper::Fun(fun_desc)) => validate_fun_call(&fun_desc, args),
                        Some(TypeWrapper::Var(_)) => front_err(format!(
                            "Error: {} is a field, not a method",
                            method_name.inner
                        )),
                        None => front_err(format!(
                            "Error: {} is not defined for class {}",
                            method_name.inner, cl_name
                        )),
                    }
                }
                Ok(_) => front_err("Error: only classes have methods".to_string()),
                Err(err) => Err(err),
            },
        };
        if let Some(new_expr) = override_expr {
            expr.inner = new_expr;
        }
        result
    }
}
