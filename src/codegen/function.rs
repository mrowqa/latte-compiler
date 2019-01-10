use codegen::class::get_size_of;
use model::{ast, ir};
use semantics::global_context::{ClassDesc, GlobalContext};
use std::collections::{HashMap, HashSet};

struct Env<'a> {
    global_ctx: &'a GlobalContext<'a>,
    class_ctx: Option<&'a ClassDesc<'a>>,
    frames: HashMap<ir::Label, EnvFrame<'a>>,
    next_proxy_frame: ir::Label,
}

struct EnvFrame<'a> {
    parent: Option<ir::Label>,
    locals: HashMap<&'a str, ir::Value>,
}

enum ValueWrapper<'a> {
    GlobalOrLocalValue(&'a ir::Value),
    #[allow(dead_code)] // todo (ext,str) remove
    ClassValue(()), // todo (ext,str)
}

struct FunctionInfoWrapper {
    ret_type: ir::Type,
    is_class_method: bool,
}

const ARGS_LABEL: ir::Label = ir::Label(std::u32::MAX);
const UNREACHABLE_LABEL: ir::Label = ir::Label(std::u32::MAX - 1);

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
            next_proxy_frame: ir::Label(std::u32::MAX - 42), // some arbitrary big label
        }
    }

    pub fn allocate_new_frame(&mut self, label: ir::Label, parent_label: ir::Label) {
        let old_frame = self.frames.insert(
            label,
            EnvFrame {
                parent: Some(parent_label),
                locals: HashMap::new(),
            },
        );
        match old_frame {
            None => (),
            Some(_) => unreachable!(), // assert
        }
    }

    pub fn add_new_local_variable(&mut self, frame: ir::Label, name: &'a str, value: ir::Value) {
        let old_val = self
            .frames
            .get_mut(&frame)
            .unwrap()
            .locals
            .insert(name, value);
        match old_val {
            None => (),
            Some(_) => unreachable!(), // assert
        }
    }

    pub fn update_existing_local_variable(
        &mut self,
        frame: ir::Label,
        name: &'a str,
        value: ir::Value,
    ) {
        let mut it = Some(frame);
        while let Some(frame) = it {
            let frame = self.frames.get_mut(&frame).unwrap();
            if frame.locals.contains_key(name) {
                frame.locals.insert(name, value);
                return;
            } else {
                it = frame.parent;
            }
        }
        unreachable!();
    }

    // proxy env should be applied later for correct visibility
    pub fn create_proxy_env(&mut self, frame_label: ir::Label) -> ir::Label {
        // loop body would modify mapping (name -> value in phi set) in condition block
        // - we want to avoid that, since body after loop needs value from this phi set
        let names = self.get_all_visible_local_variables(frame_label);
        let proxy_frame_label = self.insert_empty_proxy_frame(frame_label);
        for n in names {
            let value = match self.get_variable(frame_label, n) {
                ValueWrapper::GlobalOrLocalValue(v) => v.clone(),
                ValueWrapper::ClassValue(_) => unreachable!(),
            };
            self.frames
                .get_mut(&proxy_frame_label)
                .unwrap()
                .locals
                .insert(n, value);
        }

        proxy_frame_label
    }

    pub fn insert_empty_proxy_frame(&mut self, frame_label: ir::Label) -> ir::Label {
        let proxy_frame_label = self.next_proxy_frame;
        self.next_proxy_frame.0 -= 1;

        // block needed to satisfy borrow checker
        let parent = {
            let frame = self.frames.get_mut(&frame_label).unwrap();
            let parent = frame.parent.unwrap();
            frame.parent = Some(proxy_frame_label);
            parent
        };
        self.allocate_new_frame(proxy_frame_label, parent);

        proxy_frame_label
    }

    pub fn apply_proxy_env(&mut self, proxy: ir::Label, target: ir::Label) {
        let names = self.get_all_visible_local_variables(proxy);
        for n in names {
            let value = match self.get_variable(proxy, n) {
                ValueWrapper::GlobalOrLocalValue(v) => v.clone(),
                ValueWrapper::ClassValue(_) => unreachable!(),
            };
            self.frames
                .get_mut(&target)
                .unwrap()
                .locals
                .insert(n, value);
        }
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
            global_strings,
            env: Env::new(gctx, cctx),
            blocks: vec![],
            next_reg_num: ir::RegNum(0),
        }
    }

    pub fn generate_function_ir(mut self, fun_def: &'a ast::FunDef) -> ir::Function {
        let mut ir_args = vec![];
        for (ast_type, ast_ident) in &fun_def.args {
            let reg_num = self.get_new_reg_num();
            let arg_type = ir::Type::from_ast(&ast_type.inner);
            let arg_val = ir::Value::Register(reg_num, arg_type.clone());
            ir_args.push((reg_num, arg_type));
            self.env
                .add_new_local_variable(ARGS_LABEL, ast_ident.inner.as_ref(), arg_val);
        }

        let entry_point = self.allocate_new_block(ARGS_LABEL);
        let last_label = self.process_block(&fun_def.body, entry_point, false);
        if last_label != UNREACHABLE_LABEL {
            self.get_block(last_label)
                .body
                .push(ir::Operation::Return(None));
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
            self.add_branch1_op(parent_label, new_label);
            new_label
        } else {
            parent_label
        };

        for stmt in &block.stmts {
            use model::ast::InnerStmt::*;
            match &stmt.inner {
                Empty => (),
                Block(bl) => {
                    let end_block_label = self.process_block(bl, cur_label, true);
                    if end_block_label == UNREACHABLE_LABEL {
                        return UNREACHABLE_LABEL;
                    }
                    let cont_label = self.allocate_new_block(cur_label);
                    self.add_branch1_op(end_block_label, cont_label);
                    cur_label = cont_label;
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
                                    String | Array(_) | Class(_) => ir::Value::LitNullPtr(Some(
                                        ir::Type::from_ast(&var_type.inner),
                                    )),
                                    Null | Void => unreachable!(),
                                }
                            }
                        };
                        self.env
                            .add_new_local_variable(cur_label, var_name.inner.as_ref(), value)
                    }
                }
                Assign(lhs, rhs) => {
                    let (new_label, rhs_value) = self.process_expression(&rhs.inner, cur_label);
                    cur_label = new_label;
                    match &lhs.inner {
                        ast::InnerExpr::LitVar(var_name) => {
                            self.env
                                .update_existing_local_variable(cur_label, &var_name, rhs_value);
                        }
                        _ => {
                            let (new_label, ref_val) =
                                self.process_lvalue_ref_expression(&lhs.inner, cur_label);
                            cur_label = new_label;
                            self.get_block(cur_label)
                                .body
                                .push(ir::Operation::Store(rhs_value, ref_val));
                        }
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
                            let new_reg = self.get_new_reg_num();
                            let val_l = match self.env.get_variable(cur_label, var_name) {
                                ValueWrapper::GlobalOrLocalValue(v) => v.clone(),
                                ValueWrapper::ClassValue(_) => unimplemented!(), // todo (ext,str)
                            };
                            let val_r = ir::Value::LitInt(1);
                            self.get_block(cur_label)
                                .body
                                .push(ir::Operation::Arithmetic(new_reg, op, val_l, val_r));
                            let val_res = ir::Value::Register(new_reg, ir::Type::Int);
                            self.env
                                .update_existing_local_variable(cur_label, &var_name, val_res);
                        }
                        _ => {
                            let (new_label, ref_val) =
                                self.process_lvalue_ref_expression(&lhs.inner, cur_label);
                            cur_label = new_label;
                            let loaded_reg = self.get_new_reg_num();
                            let changed_reg = self.get_new_reg_num(); // after +/- 1
                            let body = &mut self.get_block(cur_label).body;
                            body.push(ir::Operation::Load(loaded_reg, ref_val.clone()));
                            body.push(ir::Operation::Arithmetic(
                                changed_reg,
                                op,
                                ir::Value::Register(loaded_reg, ir::Type::Int),
                                ir::Value::LitInt(1),
                            ));
                            let changed_value = ir::Value::Register(changed_reg, ir::Type::Int);
                            body.push(ir::Operation::Store(changed_value, ref_val));
                        }
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
                    return UNREACHABLE_LABEL;
                }
                Cond {
                    cond,
                    true_branch,
                    false_branch,
                } => match &cond.inner {
                    ast::InnerExpr::LitBool(true) => {
                        let end_true_label = self.process_block(true_branch, cur_label, true);
                        if end_true_label == UNREACHABLE_LABEL {
                            return UNREACHABLE_LABEL;
                        }
                        let cont_label = self.allocate_new_block(cur_label);
                        self.add_branch1_op(end_true_label, cont_label);
                        cur_label = cont_label;
                    }
                    ast::InnerExpr::LitBool(false) => match false_branch {
                        Some(bl) => {
                            let end_false_label = self.process_block(bl, cur_label, true);
                            if end_false_label == UNREACHABLE_LABEL {
                                return UNREACHABLE_LABEL;
                            }
                            let cont_label = self.allocate_new_block(cur_label);
                            self.add_branch1_op(end_false_label, cont_label);
                            cur_label = cont_label;
                        }
                        None => (),
                    },
                    expr => match false_branch {
                        None => {
                            let true_label = self.allocate_new_block(cur_label);
                            let false_label = self.allocate_new_block(cur_label); // simplifies calculation of phi function
                            let cont_label = self.allocate_new_block(cur_label);
                            self.process_expression_cond(&expr, cur_label, true_label, false_label);
                            let true_proxy_label = self.env.create_proxy_env(true_label);
                            let end_true_label = self.process_block(true_branch, true_label, false);
                            self.add_branch1_op(false_label, cont_label);
                            if end_true_label != UNREACHABLE_LABEL {
                                self.add_branch1_op(end_true_label, cont_label);
                                self.calculate_phi_set_for_if(
                                    cur_label,
                                    cont_label,
                                    (end_true_label, true_proxy_label),
                                    (false_label, false_label),
                                );
                                // phi set calculation applies proxy env properly
                            }
                            cur_label = cont_label;
                        }
                        Some(bl) => {
                            let true_label = self.allocate_new_block(cur_label);
                            let false_label = self.allocate_new_block(cur_label);
                            self.process_expression_cond(&expr, cur_label, true_label, false_label);
                            let true_proxy_label = self.env.create_proxy_env(true_label);
                            let false_proxy_label = self.env.create_proxy_env(false_label);
                            let end_true_label = self.process_block(true_branch, true_label, false);
                            let end_false_label = self.process_block(bl, false_label, false);
                            match (
                                end_true_label == UNREACHABLE_LABEL,
                                end_false_label == UNREACHABLE_LABEL,
                            ) {
                                (true, true) => return UNREACHABLE_LABEL,
                                (true, false) => {
                                    let cont_label = self.allocate_new_block(cur_label);
                                    self.add_branch1_op(end_false_label, cont_label);
                                    self.env.apply_proxy_env(false_proxy_label, cont_label);
                                    cur_label = cont_label;
                                }
                                (false, true) => {
                                    let cont_label = self.allocate_new_block(cur_label);
                                    self.add_branch1_op(end_true_label, cont_label);
                                    self.env.apply_proxy_env(true_proxy_label, cont_label);
                                    cur_label = cont_label;
                                }
                                (false, false) => {
                                    let cont_label = self.allocate_new_block(cur_label);
                                    self.add_branch1_op(end_false_label, cont_label);
                                    self.add_branch1_op(end_true_label, cont_label);
                                    self.calculate_phi_set_for_if(
                                        cur_label,
                                        cont_label,
                                        (end_true_label, true_proxy_label),
                                        (end_false_label, false_proxy_label),
                                    );
                                    // phi calculations applies proxy correctly
                                    cur_label = cont_label;
                                }
                            }
                        }
                    },
                },
                While(cond, block) => match &cond.inner {
                    ast::InnerExpr::LitBool(false) => (),
                    ast::InnerExpr::LitBool(true) => {
                        let body_label = self.allocate_new_block(cur_label);
                        let stub_info =
                            self.prepare_env_and_stub_phi_set_for_loop_cond(cur_label, body_label);
                        self.add_branch1_op(cur_label, body_label);
                        let mut end_body_label = self.process_block(block, body_label, false);
                        if end_body_label != UNREACHABLE_LABEL {
                            self.add_branch1_op(end_body_label, body_label);
                        }
                        self.finalize_phi_set_for_loop_cond(cur_label, body_label, None, stub_info);
                        return UNREACHABLE_LABEL;
                    }
                    expr => {
                        let cond_label = self.allocate_new_block(cur_label);
                        let stub_info =
                            self.prepare_env_and_stub_phi_set_for_loop_cond(cur_label, cond_label);
                        // cond_label is just fine for body_label and cond_label
                        // they will see phi functions and local variables
                        // can't be changed further in condition block
                        let body_label = self.allocate_new_block(cond_label);
                        let cont_label = self.allocate_new_block(cond_label);
                        let proxy_label = self.env.create_proxy_env(body_label);
                        self.add_branch1_op(cur_label, cond_label);
                        self.process_expression_cond(expr, cond_label, body_label, cont_label);
                        let mut end_body_label = self.process_block(block, body_label, false);
                        if end_body_label != UNREACHABLE_LABEL {
                            self.add_branch1_op(end_body_label, cond_label);
                        }
                        self.finalize_phi_set_for_loop_cond(
                            cur_label,
                            cond_label,
                            Some(proxy_label),
                            stub_info,
                        );
                        cur_label = cont_label;
                    }
                },
                // could be syntax sugar, but it introduces other problems
                ForEach {
                    iter_type,
                    iter_name,
                    array,
                    body,
                } => {
                    // calculate array
                    let (new_label, arr_val) = self.process_expression(&array.inner, cur_label);
                    cur_label = new_label;
                    let arr_type = arr_val.get_type();
                    let elem_type = ir::Type::from_ast(&iter_type.inner);

                    // calculate its length
                    let length_reg = self.get_new_reg_num();
                    let length_ref_val = self
                        .generate_calculation_of_ref_to_array_length(cur_label, arr_val.clone());
                    self.get_block(cur_label)
                        .body
                        .push(ir::Operation::Load(length_reg, length_ref_val));
                    let length_val = ir::Value::Register(length_reg, ir::Type::Int);

                    // calc base+length=end
                    let end_ptr_reg = self.get_new_reg_num();
                    self.get_block(cur_label)
                        .body
                        .push(ir::Operation::GetElementPtr(
                            end_ptr_reg,
                            elem_type.clone(),
                            arr_val.clone(),
                            length_val,
                        ));
                    let end_ptr_val = ir::Value::Register(end_ptr_reg, arr_type.clone());

                    // loop: while it<end { name=*it; it++; <body> }
                    let cond_label = self.allocate_new_block(cur_label);
                    let stub_info =
                        self.prepare_env_and_stub_phi_set_for_loop_cond(cur_label, cond_label);
                    let body_label = self.allocate_new_block(cond_label);
                    let cont_label = self.allocate_new_block(cond_label);
                    let proxy_label = self.env.create_proxy_env(body_label);
                    self.add_branch1_op(cur_label, cond_label);

                    // loop cond
                    let cur_it_reg = self.get_new_reg_num();
                    let next_it_reg = self.get_new_reg_num();
                    let cond_reg = self.get_new_reg_num();
                    let cur_it_val = ir::Value::Register(cur_it_reg, arr_type.clone());
                    let next_it_val = ir::Value::Register(next_it_reg, arr_type.clone());
                    let cond_val = ir::Value::Register(cond_reg, ir::Type::Bool);
                    self.get_block(cond_label).body.push(ir::Operation::Compare(
                        cond_reg,
                        ir::CmpOp::LT,
                        cur_it_val.clone(),
                        end_ptr_val,
                    ));
                    self.add_branch2_op(cond_label, cond_val, body_label, cont_label);

                    // loop body
                    let loaded_iter_reg = self.get_new_reg_num();
                    let loaded_iter_val = ir::Value::Register(loaded_iter_reg, elem_type.clone());
                    self.get_block(body_label)
                        .body
                        .push(ir::Operation::Load(loaded_iter_reg, cur_it_val.clone()));
                    let loop_iter_env_label = self.env.insert_empty_proxy_frame(body_label);
                    self.env.add_new_local_variable(
                        loop_iter_env_label,
                        &iter_name.inner,
                        loaded_iter_val,
                    );
                    self.get_block(body_label)
                        .body
                        .push(ir::Operation::GetElementPtr(
                            next_it_reg,
                            elem_type,
                            cur_it_val,
                            ir::Value::LitInt(1),
                        ));
                    let end_body_label = self.process_block(body, body_label, false);
                    let mut phi_vec = vec![(arr_val, cur_label)]; // for iter ptr
                    if end_body_label != UNREACHABLE_LABEL {
                        self.add_branch1_op(end_body_label, cond_label);
                        phi_vec.push((next_it_val, end_body_label));
                    }
                    self.finalize_phi_set_for_loop_cond(
                        cur_label,
                        cond_label,
                        Some(proxy_label),
                        stub_info,
                    );
                    self.get_block(cond_label)
                        .phi_set
                        .insert((cur_it_reg, arr_type, phi_vec));
                    cur_label = cont_label;
                }
                Expr(expr) => {
                    let (new_label, _) = self.process_expression(&expr.inner, cur_label);
                    cur_label = new_label;
                }
                Error => unreachable!(),
            }
        }
        // todo (optional) expressions / statements from code in comments (extract from AST)
        // todo (optional) remove empty blocks, merge paths in CFG

        cur_label
    }

    fn process_expression_cond(
        &mut self,
        expr: &ast::InnerExpr,
        cur_label: ir::Label,
        true_label: ir::Label,
        false_label: ir::Label,
    ) {
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
                self.add_branch2_op(new_label, value, true_label, false_label);
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
                if str_val == "" {
                    let str_type = ir::Type::Ptr(Box::new(ir::Type::Char));
                    (cur_label, ir::Value::LitNullPtr(Some(str_type)))
                } else {
                    let reg_num = self.get_new_reg_num();
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
            }
            LitNull(type_info) => (
                cur_label,
                ir::Value::LitNullPtr(type_info.borrow_mut().as_ref().map(ir::Type::from_ast)),
            ),
            FunCall {
                function_name,
                args,
            } => {
                let info = self.env.get_function(function_name.inner.as_ref());
                let mut args_values = vec![];
                if info.is_class_method {
                    // todo (ext,str) add "this" ptr to args
                    unimplemented!()
                }

                let mut cur_label = cur_label;
                for a in args {
                    let (new_label, value) = self.process_expression(&a.inner, cur_label);
                    cur_label = new_label;
                    // todo (ext,str) handle nulls (implicit casts)
                    args_values.push(value);
                }

                let reg_num = self.get_new_reg_num();
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
                    self.add_branch1_op(true_label, cont_label);
                    self.add_branch1_op(false_label, cont_label);
                    let new_reg = self.get_new_reg_num();
                    self.get_block(cont_label).phi_set.insert((
                        new_reg,
                        ir::Type::Bool,
                        vec![
                            (ir::Value::LitBool(true), true_label),
                            (ir::Value::LitBool(false), false_label),
                        ],
                    ));
                    (cont_label, ir::Value::Register(new_reg, ir::Type::Bool))
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
                            let new_reg = self.get_new_reg_num();
                            self.get_block(new_label)
                                .body
                                .push(ir::Operation::Arithmetic(new_reg, new_op, lhs_val, rhs_val));
                            (new_label, ir::Value::Register(new_reg, ir::Type::Int))
                        }
                        str_type @ ir::Type::Ptr(_) => {
                            let new_reg = self.get_new_reg_num();
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
                            };
                            let new_reg = self.get_new_reg_num();
                            self.get_block(new_label)
                                .body
                                .push(ir::Operation::Compare(new_reg, new_op, lhs_val, rhs_val));
                            (new_label, ir::Value::Register(new_reg, ir::Type::Bool))
                        }
                        ir::Type::Ptr(subtype) => match *subtype {
                            ir::Type::Char => {
                                let fun_name = match op {
                                    EQ => "_bltn_string_eq",
                                    NE => "_bltn_string_ne",
                                    _ => unreachable!(),
                                };
                                let new_reg = self.get_new_reg_num();
                                self.get_block(cur_label)
                                    .body
                                    .push(ir::Operation::FunctionCall(
                                        Some(new_reg),
                                        ir::Type::Bool,
                                        fun_name.to_string(),
                                        vec![lhs_val, rhs_val],
                                    ));
                                (cur_label, ir::Value::Register(new_reg, ir::Type::Bool))
                            }
                            _ => {
                                // objects & arrays
                                let cmp_op = match op {
                                    EQ => ir::CmpOp::EQ,
                                    NE => ir::CmpOp::NE,
                                    _ => unreachable!(),
                                };
                                let new_reg = self.get_new_reg_num();
                                self.get_block(cur_label).body.push(ir::Operation::Compare(
                                    new_reg, cmp_op, lhs_val, rhs_val,
                                ));
                                (cur_label, ir::Value::Register(new_reg, ir::Type::Bool))
                            }
                        },
                        ir::Type::Void | ir::Type::Char | ir::Type::Struct(_) => unreachable!(),
                    }
                }
            },
            UnaryOp(op, lhs) => match &op.inner {
                IntNeg => {
                    let (new_label, value) = self.process_expression(&lhs.inner, cur_label);
                    let new_reg = self.get_new_reg_num();
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
                    let new_reg = self.get_new_reg_num();
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
            NewArray {
                elem_type,
                elem_cnt,
            } => {
                let elem_type_ir = ir::Type::from_ast(&elem_type.inner);
                let elem_size = get_size_of(&elem_type_ir);
                let (new_label, elem_cnt_value) =
                    self.process_expression(&elem_cnt.inner, cur_label);

                let reg_num = self.get_new_reg_num();
                let casted_reg_num = self.get_new_reg_num();
                let array_type_ir = ir::Type::Ptr(Box::new(elem_type_ir));
                let body = &mut self.get_block(new_label).body;
                body.push(ir::Operation::FunctionCall(
                    Some(reg_num),
                    ir::Type::Ptr(Box::new(ir::Type::Char)),
                    "_bltn_alloc_array".to_string(),
                    vec![elem_cnt_value, ir::Value::LitInt(elem_size)],
                ));
                let void_ptr_type = ir::Type::Ptr(Box::new(ir::Type::Char));
                body.push(ir::Operation::CastPtr {
                    dst: casted_reg_num,
                    dst_type: array_type_ir.clone(),
                    src_value: ir::Value::Register(reg_num, void_ptr_type),
                });

                (
                    new_label,
                    ir::Value::Register(casted_reg_num, array_type_ir),
                )
            }
            ArrayElem { .. } => {
                // todo (ext,str) refactor; is it same as ObjField?
                let (new_label, elem_ref_value) =
                    self.process_lvalue_ref_expression(expr, cur_label);
                let new_reg = self.get_new_reg_num();
                let elem_type = match &elem_ref_value {
                    ir::Value::Register(_, ir::Type::Ptr(subtype)) => (**subtype).clone(),
                    _ => unreachable!(),
                };
                self.get_block(new_label)
                    .body
                    .push(ir::Operation::Load(new_reg, elem_ref_value));
                (new_label, ir::Value::Register(new_reg, elem_type))
            }
            NewObject(_) => unimplemented!(), // todo (ext,str)
            ObjField {
                is_obj_an_array, ..
            } => {
                let (new_label, ptr_value) = self.process_lvalue_ref_expression(expr, cur_label);
                match is_obj_an_array.get() {
                    Some(true) => {
                        let reg = self.get_new_reg_num();
                        self.get_block(cur_label)
                            .body
                            .push(ir::Operation::Load(reg, ptr_value));
                        (new_label, ir::Value::Register(reg, ir::Type::Int))
                    }
                    Some(false) => unimplemented!(), // todo (ext,str)
                    None => unreachable!(),
                }
            }
            ObjMethodCall { .. } => unimplemented!(), // todo (ext,str)
        }
    }

    fn process_lvalue_ref_expression(
        &mut self,
        expr: &ast::InnerExpr,
        cur_label: ir::Label,
    ) -> (ir::Label, ir::Value) {
        use model::ast::InnerExpr::{ArrayElem, ObjField};
        match expr {
            ArrayElem { array, index } => {
                let (new_label, array_value) = self.process_expression(&array.inner, cur_label);
                let (new_label, index_value) = self.process_expression(&index.inner, new_label);
                let new_reg = self.get_new_reg_num();
                let array_type = array_value.get_type();
                let elem_type = match &array_type {
                    ir::Type::Ptr(subtype) => (**subtype).clone(),
                    _ => unreachable!(),
                };
                self.get_block(new_label)
                    .body
                    .push(ir::Operation::GetElementPtr(
                        new_reg,
                        elem_type,
                        array_value,
                        index_value,
                    ));
                (new_label, ir::Value::Register(new_reg, array_type))
            }
            ObjField {
                obj,
                is_obj_an_array,
                ..
            } => {
                let (new_label, ptr_value) = self.process_expression(&obj.inner, cur_label);
                match is_obj_an_array.get() {
                    Some(true) => (
                        new_label,
                        self.generate_calculation_of_ref_to_array_length(new_label, ptr_value),
                    ),
                    Some(false) => unimplemented!(), // todo (ext,str)
                    None => unreachable!(),
                }
            }
            _ => unreachable!(), // we don't use store for local variables
        }
    }

    fn generate_calculation_of_ref_to_array_length(
        &mut self,
        cur_label: ir::Label,
        array_ptr: ir::Value,
    ) -> ir::Value {
        let casted_reg: ir::RegNum;
        let array_type = array_ptr.get_type();
        let elem_type = match &array_type {
            ir::Type::Ptr(subtype) => (**subtype).clone(),
            _ => unreachable!(),
        };
        let int_ptr_type = ir::Type::Ptr(Box::new(ir::Type::Int));
        match elem_type {
            ir::Type::Int => match array_ptr {
                ir::Value::Register(reg, _) => casted_reg = reg,
                _ => unreachable!(),
            },
            _ => {
                casted_reg = self.get_new_reg_num();
                self.get_block(cur_label).body.push(ir::Operation::CastPtr {
                    dst: casted_reg,
                    dst_type: int_ptr_type.clone(),
                    src_value: array_ptr,
                });
            }
        }
        let result_reg = self.get_new_reg_num();
        self.get_block(cur_label)
            .body
            .push(ir::Operation::GetElementPtr(
                result_reg,
                ir::Type::Int,
                ir::Value::Register(casted_reg, int_ptr_type.clone()),
                ir::Value::LitInt(-1),
            ));
        ir::Value::Register(result_reg, int_ptr_type)
    }

    fn calculate_phi_set_for_if(
        &mut self,
        common_pred: ir::Label,
        common_succ: ir::Label,
        (br1, br1_proxy): (ir::Label, ir::Label),
        (br2, br2_proxy): (ir::Label, ir::Label),
    ) {
        let names = self.env.get_all_visible_local_variables(common_pred);

        for name in names {
            let value0 = match self.env.get_variable(common_pred, name) {
                ValueWrapper::GlobalOrLocalValue(v) => v.clone(),
                ValueWrapper::ClassValue(_) => unreachable!(),
            };
            let value1 = match self.env.get_variable(br1_proxy, name) {
                ValueWrapper::GlobalOrLocalValue(v) => v.clone(),
                ValueWrapper::ClassValue(_) => unreachable!(),
            };
            let value2 = match self.env.get_variable(br2_proxy, name) {
                ValueWrapper::GlobalOrLocalValue(v) => v.clone(),
                ValueWrapper::ClassValue(_) => unreachable!(),
            };

            if value0 != value1 || value0 != value2 {
                let new_value = if value1 == value2 {
                    value1 // no need to emit phi function, just update environment
                } else {
                    let reg_num = self.get_new_reg_num();
                    let reg_type = value1.get_type();
                    self.get_block(common_succ).phi_set.insert((
                        reg_num,
                        reg_type.clone(),
                        vec![(value1, br1), (value2, br2)],
                    ));
                    ir::Value::Register(reg_num, reg_type)
                };
                self.env
                    .update_existing_local_variable(common_succ, name, new_value);
            }
        }
    }

    // must be called before processing an expression (it updates environment)
    fn prepare_env_and_stub_phi_set_for_loop_cond(
        &mut self,
        pred_label: ir::Label,
        cond_label: ir::Label,
    ) -> Vec<(&'a str, ir::Value, ir::Value)> {
        let names = self.env.get_all_visible_local_variables(pred_label);
        let mut stub_info = vec![];

        for name in names {
            let value = match self.env.get_variable(pred_label, name) {
                ValueWrapper::GlobalOrLocalValue(v) => v.clone(),
                ValueWrapper::ClassValue(_) => unreachable!(),
            };
            let reg_num = self.get_new_reg_num();
            let phi_value = ir::Value::Register(reg_num, value.get_type());
            stub_info.push((name, value, phi_value.clone()));
            self.env
                .update_existing_local_variable(cond_label, name, phi_value);
        }

        stub_info
    }

    // must be called after processing cond and body blocks
    fn finalize_phi_set_for_loop_cond(
        &mut self,
        pred_label: ir::Label,
        cond_label: ir::Label,
        proxy_label: Option<ir::Label>,
        stub_info: Vec<(&'a str, ir::Value, ir::Value)>,
    ) {
        let end_body_label = {
            let preds = &self.get_block(cond_label).predecessors;
            if preds.len() == 1 {
                UNREACHABLE_LABEL
            } else {
                assert_eq!(preds.len(), 2);
                if preds[0] != pred_label {
                    preds[0]
                } else {
                    preds[1]
                }
            }
        };

        for (name, value1, phi_value) in stub_info {
            let mut phi_vec = vec![(value1, pred_label)];
            if end_body_label != UNREACHABLE_LABEL {
                // this is really tricky; we need to lookup proxy_label, not
                // end_body_label, so we will not confuse new variables
                // defined in body loop which shadows original ones
                let proxy_label = proxy_label.unwrap();
                let value2 = match self.env.get_variable(proxy_label, name) {
                    ValueWrapper::GlobalOrLocalValue(v) => v.clone(),
                    ValueWrapper::ClassValue(_) => unreachable!(),
                };
                phi_vec.push((value2, end_body_label));
            }
            let (reg_num, reg_type) = match phi_value {
                ir::Value::Register(reg_num, reg_type) => (reg_num, reg_type),
                _ => unreachable!(),
            };
            self.get_block(cond_label)
                .phi_set
                .insert((reg_num, reg_type, phi_vec));
        }
    }

    fn allocate_new_block(&mut self, parent_env_label: ir::Label) -> ir::Label {
        let label = ir::Label(self.blocks.len() as u32);
        self.blocks.push(ir::Block {
            label,
            phi_set: HashSet::new(),
            predecessors: vec![],
            body: vec![],
        });
        self.env.allocate_new_frame(label, parent_env_label);
        label
    }

    fn add_branch1_op(&mut self, src: ir::Label, dst: ir::Label) {
        self.get_block(src).body.push(ir::Operation::Branch1(dst));
        self.get_block(dst).predecessors.push(src);
    }

    fn add_branch2_op(&mut self, src: ir::Label, cond: ir::Value, br1: ir::Label, br2: ir::Label) {
        self.get_block(src)
            .body
            .push(ir::Operation::Branch2(cond, br1, br2));
        self.get_block(br1).predecessors.push(src);
        self.get_block(br2).predecessors.push(src);
    }

    fn get_new_reg_num(&mut self) -> ir::RegNum {
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
