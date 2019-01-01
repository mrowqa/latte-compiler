use model::{ast, ir};
use semantics::global_context::{ClassDesc, GlobalContext};
use std::collections::{HashMap, HashSet};

struct Env<'a> {
    global_ctx: &'a GlobalContext<'a>,
    class_ctx: Option<&'a ClassDesc<'a>>,
    frames: HashMap<ir::Label, EnvFrame<'a>>,
}

struct EnvFrame<'a> {
    parent: Option<ir::Label>,
    locals: HashMap<&'a str, ir::Value>,
}

enum ValueWrapper<'a> {
    GlobalOrLocalValue(&'a ir::Value),
    #[allow(dead_code)] // todo (ext) remove
    ClassValue(()), // todo (ext)
}

struct FunctionInfoWrapper {
    ret_type: ir::Type,
    is_class_method: bool,
}

const ARGS_LABEL: ir::Label = ir::Label(std::u32::MAX);

impl<'a> Env<'a> {
    pub fn new(gctx: &'a GlobalContext<'a>, cctx: Option<&'a ClassDesc<'a>>) -> Env<'a> {
        let mut frames = HashMap::new();
        frames.insert(
            ARGS_LABEL,
            EnvFrame {
                parent: None,
                locals: HashMap::new(),
            },
        );
        Env {
            global_ctx: gctx,
            class_ctx: cctx,
            frames,
        }
    }

    pub fn allocate_new_frame(&mut self, label: ir::Label, parent_label: ir::Label) {
        self.frames.insert(
            label,
            EnvFrame {
                parent: Some(parent_label),
                locals: HashMap::new(),
            },
        );
    }

    pub fn update_local_variable(&mut self, frame: ir::Label, name: &'a str, value: ir::Value) {
        self.frames
            .get_mut(&frame)
            .unwrap()
            .locals
            .insert(name, value);
    }

    pub fn get_variable(&self, frame: ir::Label, name: &'a str) -> ValueWrapper {
        let mut it = Some(frame);

        while let Some(frame_no) = it {
            let frame = &self.frames[&frame_no];
            match frame.locals.get(name) {
                Some(v) => return ValueWrapper::GlobalOrLocalValue(v),
                None => it = frame.parent,
            }
        }

        // todo (ext) get class var
        unimplemented!()
        // if let Some(cctx) = ctx.class_ctx {
        //     match cctx.get_item(ctx.global_ctx, name) {
        //         Some(TypeWrapper::Var(t)) => return Ok(t.inner.clone()),
        //         _ => unreachable!(),
        //     }
        // }
    }

    pub fn get_function(&self, name: &str) -> FunctionInfoWrapper {
        if let Some(_cctx) = self.class_ctx {
            // todo (ext) class method
            unimplemented!()
            // match cctx.get_item(ctx.global_ctx, name) {
            //     Some(TypeWrapper::Fun(f)) => return Ok(f),
            //     _ => // ask global ctx
            // }
        }

        let desc = self.global_ctx.get_function_description(name).unwrap();
        FunctionInfoWrapper {
            ret_type: ir::Type::from_ast(&desc.ret_type.inner),
            is_class_method: false,
        }
    }

    fn get_all_visible_local_variables(&self, frame: ir::Label) -> HashSet<&'a str> {
        let mut names = HashSet::new();
        let mut it = Some(frame);

        while let Some(frame_no) = it {
            let frame = &self.frames[&frame_no];
            names.extend(frame.locals.keys());
            it = frame.parent;
        }

        names
    }
}

pub struct FunctionCodeGen<'a> {
    // global_ctx: &'a GlobalContext<'a>,
    // class_ctx: Option<&'a ClassDesc<'a>>,
    global_strings: &'a mut HashMap<String, ir::GlobalStrNum>,
    env: Env<'a>,
    blocks: Vec<ir::Block>,
    next_reg_num: ir::RegNum,
}

impl<'a> FunctionCodeGen<'a> {
    pub fn new(
        gctx: &'a GlobalContext<'a>,
        cctx: Option<&'a ClassDesc<'a>>,
        global_strings: &'a mut HashMap<String, ir::GlobalStrNum>,
    ) -> Self {
        FunctionCodeGen {
            // class_ctx: cctx,
            // global_ctx: gctx,
            global_strings,
            env: Env::new(gctx, cctx),
            blocks: vec![],
            next_reg_num: ir::RegNum(0),
        }
    }

    pub fn generate_function_ir(mut self, fun_def: &'a ast::FunDef) -> ir::Function {
        let mut ir_args = vec![];
        for (ast_type, ast_ident) in &fun_def.args {
            let reg_num = self.fresh_reg_num();
            let arg_type = ir::Type::from_ast(&ast_type.inner);
            let arg_val = ir::Value::Register(reg_num, arg_type.clone());
            ir_args.push((reg_num, arg_type));
            self.env
                .update_local_variable(ARGS_LABEL, ast_ident.inner.as_ref(), arg_val);
        }

        let entry_point = self.allocate_new_block(ARGS_LABEL);
        let last_label = self.process_block(&fun_def.body, entry_point, false);
        match fun_def.ret_type.inner {
            ast::InnerType::Void => {
                let last_block = self.get_block(last_label);
                match last_block.body.last() {
                    Some(ir::Operation::Return(None)) => (),
                    _ => {
                        last_block.body.push(ir::Operation::Return(None));
                    }
                }
            }
            _ => (),
        }

        ir::Function {
            ret_type: ir::Type::from_ast(&fun_def.ret_type.inner),
            name: fun_def.name.inner.clone(),
            args: ir_args,
            blocks: self.blocks,
        }
    }

    fn process_block(
        &mut self,
        block: &'a ast::Block,
        parent_label: ir::Label,
        allocate_new_label: bool,
    ) -> ir::Label {
        let mut cur_label = if allocate_new_label {
            let new_label = self.allocate_new_block(parent_label);
            self.get_block(parent_label)
                .body
                .push(ir::Operation::Branch1(new_label));
            new_label
        } else {
            parent_label
        };

        for stmt in &block.stmts {
            use model::ast::InnerStmt::*;
            match &stmt.inner {
                Empty => (),
                Block(bl) => {
                    cur_label = self.process_block(bl, cur_label, true);
                }
                Decl {
                    var_type,
                    var_items,
                } => {
                    for (var_name, var_init) in var_items {
                        let value = match var_init {
                            Some(expr) => {
                                let (new_label, value) =
                                    self.process_expression(&expr.inner, cur_label);
                                cur_label = new_label;
                                value
                            }
                            None => {
                                use model::ast::InnerType::*;
                                match &var_type.inner {
                                    Int => ir::Value::LitInt(0),
                                    Bool => ir::Value::LitBool(false),
                                    String => self.get_global_string(""),
                                    Array(_) | Class(_) => ir::Value::LitNullPtr,
                                    Null | Void => unreachable!(),
                                }
                            }
                        };
                        // todo (ext) handle nulls
                        self.env
                            .update_local_variable(cur_label, var_name.inner.as_ref(), value)
                    }
                }
                Assign(lhs, rhs) => {
                    // todo (ext) refactor assign/incr/decr somehow
                    let (new_label, value) = self.process_expression(&rhs.inner, cur_label);
                    cur_label = new_label;
                    match &lhs.inner {
                        ast::InnerExpr::LitVar(var_name) => {
                            self.env.update_local_variable(cur_label, &var_name, value);
                        }
                        _ => unimplemented!(), // todo (ext)
                    };
                }
                Incr(lhs) | Decr(lhs) => {
                    let op = match &stmt.inner {
                        Incr(_) => ir::ArithOp::Add,
                        Decr(_) => ir::ArithOp::Sub,
                        _ => unreachable!(),
                    };
                    match &lhs.inner {
                        ast::InnerExpr::LitVar(var_name) => {
                            let new_reg = self.fresh_reg_num();
                            let val_l = match self.env.get_variable(cur_label, var_name) {
                                ValueWrapper::GlobalOrLocalValue(v) => v.clone(),
                                ValueWrapper::ClassValue(_) => unimplemented!(), // todo (ext)
                            };
                            let val_r = ir::Value::LitInt(1);
                            self.get_block(cur_label)
                                .body
                                .push(ir::Operation::Arithmetic(new_reg, op, val_l, val_r));
                            let val_res = ir::Value::Register(new_reg, ir::Type::Int);
                            self.env
                                .update_local_variable(cur_label, &var_name, val_res);
                        }
                        _ => unimplemented!(), // todo (ext)
                    };
                }
                Ret(opt_expr) => {
                    let mut opt_value = opt_expr.as_ref().map(|expr| {
                        let (new_label, value) = self.process_expression(&expr.inner, cur_label);
                        cur_label = new_label;
                        value
                    });
                    opt_value = match opt_value {
                        Some(ir::Value::Register(_, ir::Type::Void)) => None,
                        _ => opt_value,
                    };
                    self.get_block(cur_label)
                        .body
                        .push(ir::Operation::Return(opt_value));
                }
                Cond {
                    cond,
                    true_branch,
                    false_branch,
                } => match &cond.inner {
                    ast::InnerExpr::LitBool(true) => {
                        cur_label = self.process_block(true_branch, cur_label, true);
                    }
                    ast::InnerExpr::LitBool(false) => match false_branch {
                        Some(bl) => cur_label = self.process_block(bl, cur_label, true),
                        None => (),
                    },
                    expr => {
                        // todo readme <- optimization
                        let true_label = self.allocate_new_block(cur_label);
                        let false_label = self.allocate_new_block(cur_label);
                        self.process_expression_cond(&expr, cur_label, true_label, false_label);
                        let mut cont_label: ir::Label;
                        let end_true_label = self.process_block(true_branch, true_label, false);
                        let end_false_label = match false_branch {
                            Some(bl) => {
                                let end_false_label = self.process_block(bl, false_label, false);
                                cont_label = self.allocate_new_block(cur_label);
                                self.get_block(end_false_label)
                                    .body
                                    .push(ir::Operation::Branch1(cont_label));
                                end_false_label
                            }
                            None => {
                                cont_label = false_label;
                                cur_label
                            }
                        };
                        self.get_block(end_true_label)
                            .body
                            .push(ir::Operation::Branch1(cont_label));
                        self.calculate_phi_set(
                            cur_label,
                            end_true_label,
                            end_false_label,
                            cont_label,
                        );
                        cur_label = cont_label;
                    }
                },
                While(cond, block) => match &cond.inner {
                    //ast::InnerExpr::LitBool(true) => {} // todo (optional) some UNREACHABLE_LABEL (?) for not generating dead code? or unreachable llvm instruction?
                    ast::InnerExpr::LitBool(false) => (),
                    expr => {
                        // todo bug (circular dependency):
                        // cond and body must be processed to calculate phi set
                        // phi set must be calculated to have valid registers for cond and body
                        let cond_label = self.allocate_new_block(cur_label);
                        // cond_label is just fine for body_label and cond_label
                        // they will see phi functions and local variables
                        // can't be changed further in condition block
                        let body_label = self.allocate_new_block(cond_label);
                        let cont_label = self.allocate_new_block(cond_label);
                        self.get_block(cur_label)
                            .body
                            .push(ir::Operation::Branch1(cond_label));
                        self.process_expression_cond(expr, cond_label, body_label, cont_label);
                        let end_body_label = self.process_block(block, body_label, false);
                        self.get_block(end_body_label)
                            .body
                            .push(ir::Operation::Branch1(cond_label));
                        self.calculate_phi_set(cur_label, cur_label, end_body_label, cond_label);
                        cur_label = cont_label;
                    }
                },
                ForEach { .. } => unimplemented!(), // todo (ext)
                Expr(expr) => {
                    let (new_label, _) = self.process_expression(&expr.inner, cur_label);
                    cur_label = new_label;
                }
                Error => unreachable!(),
            }
        }
        // todo reorder blocks for better LLVM linear code? (note: add info to README)

        cur_label
    }

    fn process_expression_cond(
        &mut self,
        expr: &ast::InnerExpr,
        cur_label: ir::Label,
        true_label: ir::Label,
        false_label: ir::Label,
    ) {
        // todo add to readme, ! optimisation, if and while optimizations
        use model::ast::{BinaryOp::*, InnerExpr::*, InnerUnaryOp::*};
        match expr {
            BinaryOp(lhs, And, rhs) => {
                let mid_label = self.allocate_new_block(cur_label);
                self.process_expression_cond(&lhs.inner, cur_label, mid_label, false_label);
                self.process_expression_cond(&rhs.inner, mid_label, true_label, false_label);
            }
            BinaryOp(lhs, Or, rhs) => {
                let mid_label = self.allocate_new_block(cur_label);
                self.process_expression_cond(&lhs.inner, cur_label, true_label, mid_label);
                self.process_expression_cond(&rhs.inner, mid_label, true_label, false_label);
            }
            UnaryOp(ast::ItemWithSpan { inner: BoolNeg, .. }, lhs) => {
                self.process_expression_cond(&lhs.inner, cur_label, false_label, true_label);
            }
            _ => {
                let (new_label, value) = self.process_expression(&expr, cur_label);
                self.get_block(new_label).body.push(ir::Operation::Branch2(
                    value,
                    true_label,
                    false_label,
                ));
            }
        }
    }

    fn process_expression(
        &mut self,
        expr: &ast::InnerExpr,
        cur_label: ir::Label,
    ) -> (ir::Label, ir::Value) {
        use model::ast::{BinaryOp::*, InnerExpr::*, InnerUnaryOp::*};
        match expr {
            LitVar(var_name) => {
                match self.env.get_variable(cur_label, var_name) {
                    ValueWrapper::GlobalOrLocalValue(value) => (cur_label, value.clone()),
                    ValueWrapper::ClassValue(_) => unimplemented!(), // todo (ext)
                }
            }
            LitInt(int_val) => (cur_label, ir::Value::LitInt(*int_val)),
            LitBool(bool_val) => (cur_label, ir::Value::LitBool(*bool_val)),
            LitStr(str_val) => {
                let reg_num = self.fresh_reg_num();
                let str_ir_val = self.get_global_string(str_val);
                match str_ir_val {
                    ir::Value::GlobalRegister(str_num) => {
                        self.get_block(cur_label)
                            .body
                            .push(ir::Operation::CastGlobalString(
                                reg_num,
                                str_val.len() + 1,
                                str_num,
                            ))
                    }
                    _ => unreachable!(),
                }
                let str_type = ir::Type::Ptr(Box::new(ir::Type::Char));
                let casted_val = ir::Value::Register(reg_num, str_type);
                (cur_label, casted_val)
            }
            LitNull => (cur_label, ir::Value::LitNullPtr),
            FunCall {
                function_name,
                args,
            } => {
                let info = self.env.get_function(function_name.inner.as_ref());
                let mut args_values = vec![];
                if info.is_class_method {
                    // todo (ext) add "this" ptr to args
                    unimplemented!()
                }

                let mut cur_label = cur_label;
                for a in args {
                    let (new_label, value) = self.process_expression(&a.inner, cur_label);
                    cur_label = new_label;
                    // todo (ext) handle nulls (implicit casts)
                    args_values.push(value);
                }

                let reg_num = self.fresh_reg_num();
                let op_reg_num = match info.ret_type {
                    ir::Type::Void => None,
                    _ => Some(reg_num),
                };

                self.get_block(cur_label)
                    .body
                    .push(ir::Operation::FunctionCall(
                        op_reg_num,
                        info.ret_type.clone(),
                        function_name.inner.clone(),
                        args_values,
                    ));
                (cur_label, ir::Value::Register(reg_num, info.ret_type))
            }
            BinaryOp(lhs, op, rhs) => match op {
                And | Or => {
                    let true_label = self.allocate_new_block(cur_label);
                    let false_label = self.allocate_new_block(cur_label);
                    self.process_expression_cond(&expr, cur_label, true_label, false_label);
                    let cont_label = self.allocate_new_block(cur_label);
                    self.get_block(true_label)
                        .body
                        .push(ir::Operation::Branch1(cont_label));
                    self.get_block(false_label)
                        .body
                        .push(ir::Operation::Branch1(cont_label));
                    let new_reg = self.fresh_reg_num();
                    self.get_block(cont_label).phi_set.insert((
                        new_reg,
                        ir::Type::Bool,
                        vec![
                            (ir::Value::LitBool(true), true_label),
                            (ir::Value::LitBool(false), false_label),
                        ],
                    ));
                    (cur_label, ir::Value::Register(new_reg, ir::Type::Bool))
                }
                Add | Sub | Mul | Div | Mod => {
                    let (new_label, lhs_val) = self.process_expression(&lhs.inner, cur_label);
                    let (new_label, rhs_val) = self.process_expression(&rhs.inner, new_label);
                    match lhs_val.get_type() {
                        ir::Type::Int => {
                            let new_op = match op {
                                Add => ir::ArithOp::Add,
                                Sub => ir::ArithOp::Sub,
                                Mul => ir::ArithOp::Mul,
                                Div => ir::ArithOp::Div,
                                Mod => ir::ArithOp::Mod,
                                _ => unreachable!(),
                            };
                            let new_reg = self.fresh_reg_num();
                            self.get_block(new_label)
                                .body
                                .push(ir::Operation::Arithmetic(new_reg, new_op, lhs_val, rhs_val));
                            (new_label, ir::Value::Register(new_reg, ir::Type::Int))
                        }
                        str_type @ ir::Type::Ptr(_) => {
                            let new_reg = self.fresh_reg_num();
                            self.get_block(new_label)
                                .body
                                .push(ir::Operation::FunctionCall(
                                    Some(new_reg),
                                    str_type.clone(),
                                    "_bltn_string_concat".to_string(),
                                    vec![lhs_val, rhs_val],
                                ));
                            (new_label, ir::Value::Register(new_reg, str_type))
                        }
                        _ => unreachable!(),
                    }
                }
                LT | LE | GT | GE | EQ | NE => {
                    let (new_label, lhs_val) = self.process_expression(&lhs.inner, cur_label);
                    let (new_label, rhs_val) = self.process_expression(&rhs.inner, new_label);
                    match lhs_val.get_type() {
                        ir::Type::Int | ir::Type::Bool => {
                            let new_op = match op {
                                LT => ir::CmpOp::LT,
                                LE => ir::CmpOp::LE,
                                GT => ir::CmpOp::GT,
                                GE => ir::CmpOp::GE,
                                EQ => ir::CmpOp::EQ,
                                NE => ir::CmpOp::NE,
                                _ => unreachable!(),
                            }; // todo test &&, || and !
                            let new_reg = self.fresh_reg_num();
                            self.get_block(new_label)
                                .body
                                .push(ir::Operation::Compare(new_reg, new_op, lhs_val, rhs_val));
                            (new_label, ir::Value::Register(new_reg, ir::Type::Bool))
                        }
                        ir::Type::Ptr(subtype) => {
                            match *subtype {
                                ir::Type::Char => {
                                    // todo test compare strings (after fixing while-phi set problem)
                                    let fun_name = match op {
                                        EQ => "_bltn_string_eq",
                                        NE => "_bltn_string_ne",
                                        _ => unreachable!(),
                                    };
                                    let new_reg = self.fresh_reg_num();
                                    self.get_block(cur_label).body.push(
                                        ir::Operation::FunctionCall(
                                            Some(new_reg),
                                            ir::Type::Bool,
                                            fun_name.to_string(),
                                            vec![lhs_val, rhs_val],
                                        ),
                                    );
                                    (cur_label, ir::Value::Register(new_reg, ir::Type::Bool))
                                }
                                _ => {
                                    // todo (ext) comparing nulls with classes and arrays
                                    unimplemented!()
                                }
                            }
                        }
                        ir::Type::Void | ir::Type::Char | ir::Type::Struct(_) => unreachable!(),
                    }
                }
            },
            UnaryOp(op, lhs) => match &op.inner {
                IntNeg => {
                    let (new_label, value) = self.process_expression(&lhs.inner, cur_label);
                    let new_reg = self.fresh_reg_num();
                    self.get_block(new_label)
                        .body
                        .push(ir::Operation::Arithmetic(
                            new_reg,
                            ir::ArithOp::Sub,
                            ir::Value::LitInt(0),
                            value,
                        ));
                    (new_label, ir::Value::Register(new_reg, ir::Type::Int))
                }
                BoolNeg => {
                    let (new_label, value) = self.process_expression(&lhs.inner, cur_label);
                    let new_reg = self.fresh_reg_num();
                    self.get_block(new_label)
                        .body
                        .push(ir::Operation::Arithmetic(
                            new_reg,
                            ir::ArithOp::Sub,
                            ir::Value::LitBool(true),
                            value,
                        ));
                    (new_label, ir::Value::Register(new_reg, ir::Type::Bool))
                }
            },
            NewArray { .. } => unimplemented!(),  // todo (ext)
            ArrayElem { .. } => unimplemented!(), // todo (ext)
            NewObject(_) => unimplemented!(),     // todo (ext)
            ObjField { .. } => unimplemented!(),  // todo (ext)
            ObjMethodCall { .. } => unimplemented!(), // todo (ext)
        }
    }

    fn calculate_phi_set(
        &mut self,
        common_pred: ir::Label,
        br1: ir::Label,
        br2: ir::Label,
        common_succ: ir::Label,
    ) {
        let names1 = self.env.get_all_visible_local_variables(br1);
        let names2 = self.env.get_all_visible_local_variables(br2);
        let mut phi_set = HashSet::new(); // needed to satisfy borrow checker

        for name in names2.union(&names1) {
            let value0 = match self.env.get_variable(common_pred, name) {
                ValueWrapper::GlobalOrLocalValue(v) => v,
                ValueWrapper::ClassValue(_) => unreachable!(),
            };
            let value1 = match self.env.get_variable(br1, name) {
                ValueWrapper::GlobalOrLocalValue(v) => v,
                ValueWrapper::ClassValue(_) => unreachable!(),
            };
            let value2 = match self.env.get_variable(br2, name) {
                ValueWrapper::GlobalOrLocalValue(v) => v,
                ValueWrapper::ClassValue(_) => unreachable!(),
            };

            if value0 != value1 || value0 != value2 {
                phi_set.insert((name, value1.clone(), value2.clone()));
            }
        }

        // todo (ext) readme mention handling nulls - not trivial
        for (name, value1, value2) in phi_set {
            let reg_num = self.fresh_reg_num();
            let reg_type = value1.get_type(); // todo (ext) handle nulls somehow
            self.get_block(common_succ).phi_set.insert((
                reg_num,
                reg_type.clone(),
                vec![(value1, br1), (value2, br2)],
            ));
            self.env.update_local_variable(
                common_succ,
                name,
                ir::Value::Register(reg_num, reg_type),
            );
        }
    }

    fn allocate_new_block(&mut self, parent_label: ir::Label) -> ir::Label {
        let label = ir::Label(self.blocks.len() as u32);
        self.blocks.push(ir::Block {
            label,
            phi_set: HashSet::new(),
            body: vec![],
        });
        self.env.allocate_new_frame(label, parent_label);
        label
    }

    fn fresh_reg_num(&mut self) -> ir::RegNum {
        let ir::RegNum(no) = self.next_reg_num;
        self.next_reg_num.0 += 1;
        ir::RegNum(no)
    }

    fn get_block(&mut self, label: ir::Label) -> &mut ir::Block {
        &mut self.blocks[label.0 as usize]
    }

    fn get_global_string(&mut self, string: &str) -> ir::Value {
        if let Some(reg) = self.global_strings.get(string) {
            return ir::Value::GlobalRegister(*reg);
        }

        let reg = ir::GlobalStrNum(self.global_strings.len() as u32);
        self.global_strings.insert(string.to_string(), reg);
        ir::Value::GlobalRegister(reg)
    }
}
