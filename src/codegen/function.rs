use model::{ast, ir};
use semantics::global_context::{ClassDesc, GlobalContext};
use std::collections::{HashMap, HashSet};

struct Env<'a> {
    #[allow(dead_code)] // todo remove
    class_ctx: Option<&'a ClassDesc<'a>>,
    #[allow(dead_code)] // todo remove
    global_ctx: &'a GlobalContext<'a>,
    frames: HashMap<ir::Label, EnvFrame<'a>>,
}

struct EnvFrame<'a> {
    #[allow(dead_code)] // todo remove
    parent: Option<ir::Label>,
    locals: HashMap<&'a str, ir::Value>,
}

// enum ValueWrapper<'a> {
//     GlobalOrLocalValue(&'a ir::Value),
//     ClassValue(()), // todo
// }

// struct FunctionValueWrapper {
//     ret_type: ir::Type,
//     is_class_method: bool,
// }

const ARGS_LABEL: ir::Label = ir::Label(std::u32::MAX);

impl<'a> Env<'a> {
    pub fn new(cctx: Option<&'a ClassDesc<'a>>, gctx: &'a GlobalContext<'a>) -> Env<'a> {
        let mut frames = HashMap::new();
        frames.insert(
            ARGS_LABEL,
            EnvFrame {
                parent: None,
                locals: HashMap::new(),
            },
        );
        Env {
            class_ctx: cctx,
            global_ctx: gctx,
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

    pub fn add_local_variable(&mut self, frame: ir::Label, name: &'a str, value: ir::Value) {
        self.frames
            .get_mut(&frame)
            .unwrap()
            .locals
            .insert(name, value);
    }

    // pub fn get_variable(&self, name: &str) -> ValueWrapper {
    //     match self {
    //         Env::Root { .. } => {
    //             // todo
    //             unimplemented!()
    //             // if let Some(cctx) = ctx.class_ctx {
    //             //     match cctx.get_item(ctx.global_ctx, name) {
    //             //         Some(TypeWrapper::Var(t)) => return Ok(t.inner.clone()),
    //             //         _ => unreachable!(),
    //             //     }
    //             // }
    //         }
    //         Env::Nested { locals, parent } => match locals.get(name) {
    //             Some(v) => ValueWrapper::GlobalOrLocalValue(v),
    //             None => parent.get_variable(name),
    //         },
    //     }
    // }

    // pub fn get_function(&self, name: &str) -> FunctionValueWrapper {
    //     match self {
    //         Env::Root { .. } => {
    //             // todo
    //             unimplemented!()
    //             // if let Some(cctx) = ctx.class_ctx {
    //             //     match cctx.get_item(ctx.global_ctx, name) {
    //             //         Some(TypeWrapper::Fun(f)) => return Ok(f),
    //             //         _ => // ask global ctx
    //             //     }
    //             // }
    //         }
    //         Env::Nested { parent, .. } => parent.get_function(name),
    //     }
    // }
}

pub struct FunctionCodeGen<'a> {
    // class_ctx: Option<&'a ClassDesc<'a>>,
    // global_ctx: &'a GlobalContext<'a>,
    env: Env<'a>,
    blocks: Vec<ir::Block>,
    next_reg_num: ir::RegNum,
}

impl<'a> FunctionCodeGen<'a> {
    pub fn new(cctx: Option<&'a ClassDesc<'a>>, gctx: &'a GlobalContext<'a>) -> Self {
        FunctionCodeGen {
            // class_ctx: cctx,
            // global_ctx: gctx,
            env: Env::new(cctx, gctx),
            blocks: vec![],
            next_reg_num: ir::RegNum(0),
        }
    }

    pub fn generate_function_ir(mut self, fun_def: &'a ast::FunDef) -> ir::Function {
        let mut ir_args = vec![];
        for (ast_type, ast_ident) in &fun_def.args {
            let arg_type = ir::Type::from_ast(&ast_type.inner);
            let arg_val = ir::Value::Register(self.fresh_reg_num(), arg_type.clone());
            ir_args.push((arg_type, ast_ident.inner.clone()));
            self.env
                .add_local_variable(ARGS_LABEL, ast_ident.inner.as_ref(), arg_val);
        }

        let entry_point = self.allocate_new_block(ARGS_LABEL);
        self.process_block(&fun_def.body, entry_point, false);

        ir::Function {
            ret_type: ir::Type::from_ast(&fun_def.ret_type.inner),
            name: fun_def.name.inner.clone(),
            args: ir_args,
            blocks: self.blocks,
        }
    }

    fn process_block(
        &mut self,
        block: &ast::Block,
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
                        let true_label = self.allocate_new_block(cur_label);
                        let false_label = self.allocate_new_block(cur_label);
                        self.process_expression_cond(&expr, cur_label, true_label, false_label);
                        self.process_block(true_branch, true_label, false);
                        match false_branch {
                            Some(bl) => {
                                self.process_block(bl, false_label, false);
                                cur_label = self.allocate_new_block(cur_label);
                                self.get_block(false_label)
                                    .body
                                    .push(ir::Operation::Branch1(cur_label));
                            }
                            None => cur_label = false_label,
                        }
                        self.get_block(true_label)
                            .body
                            .push(ir::Operation::Branch1(cur_label));
                        // todo fill in phi set!
                    }
                },
                While(cond, block) => match &cond.inner {
                    //ast::InnerExpr::LitBool(true) => {} // todo some UNREACHABLE_LABEL (?) for not generating dead code?
                    ast::InnerExpr::LitBool(false) => (),
                    expr => {
                        let cond_label = self.allocate_new_block(cur_label);
                        let body_label = self.allocate_new_block(cur_label);
                        let cont_label = self.allocate_new_block(cur_label);
                        self.get_block(cur_label)
                            .body
                            .push(ir::Operation::Branch1(cond_label));
                        self.process_expression_cond(expr, cond_label, body_label, cont_label);
                        let end_body_label = self.process_block(block, body_label, false);
                        self.get_block(end_body_label)
                            .body
                            .push(ir::Operation::Branch1(cont_label));
                        cur_label = cont_label;
                        // todo fill in phi set
                    }
                },

                // todo expr
                _ => unimplemented!(), // todo
            }
        }
        // todo reorder blocks for better LLVM linear code?

        cur_label
    }

    fn process_expression_cond(
        &mut self,
        expr: &ast::InnerExpr,
        cur_label: ir::Label,
        true_label: ir::Label,
        false_label: ir::Label,
    ) {
        // todo add to readme, ! optimisation, if and while optimisations
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
        mut cur_label: ir::Label,
    ) -> (ir::Label, ir::Value) {
        use model::ast::{BinaryOp::*, InnerExpr::*, InnerUnaryOp::*};
        match expr {
            LitVar(_) => unimplemented!(), // todo
            LitInt(int_val) => (cur_label, ir::Value::LitInt(*int_val)),
            LitBool(bool_val) => (cur_label, ir::Value::LitBool(*bool_val)),
            LitStr(_) => unimplemented!(), // todo
            LitNull => (cur_label, ir::Value::LitNullPtr),
            FunCall { .. } => unimplemented!(), // todo
            BinaryOp(_lhs, op, _rhs) => {
                match op {
                    And | Or => {
                        let true_label = self.allocate_new_block(cur_label);
                        let false_label = self.allocate_new_block(cur_label);
                        self.process_expression_cond(&expr, cur_label, true_label, false_label);
                        cur_label = self.allocate_new_block(cur_label);
                        self.get_block(true_label)
                            .body
                            .push(ir::Operation::Branch1(cur_label));
                        self.get_block(false_label)
                            .body
                            .push(ir::Operation::Branch1(cur_label));
                        let new_reg = self.fresh_reg_num();
                        self.get_block(cur_label).phi_set.insert((
                            new_reg,
                            ir::Type::Bool,
                            vec![
                                (ir::Value::LitBool(true), true_label),
                                (ir::Value::LitBool(false), false_label),
                            ],
                        ));
                        (cur_label, ir::Value::Register(new_reg, ir::Type::Bool))
                    }
                    _ => unimplemented!(), // todo
                }
            }
            UnaryOp(op, _lhs) => match &op.inner {
                IntNeg => unimplemented!(),  // todo
                BoolNeg => unimplemented!(), // todo | 1-x or phi_set?
            },
            NewArray { .. } => unimplemented!(),      // todo
            ArrayElem { .. } => unimplemented!(),     // todo
            NewObject(_) => unimplemented!(),         // todo
            ObjField { .. } => unimplemented!(),      // todo
            ObjMethodCall { .. } => unimplemented!(), // todo
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
}
