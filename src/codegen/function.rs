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
    blocks: Vec<ir::Block>, // or hash map (borrow checker problems?) todo?
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
            let arg_val = ir::Value::Register(arg_type.clone(), self.fresh_reg_num());
            ir_args.push((arg_type, ast_ident.inner.clone()));
            self.env
                .add_local_variable(ARGS_LABEL, ast_ident.inner.as_ref(), arg_val);
        }

        self.process_block(&fun_def.body, ARGS_LABEL);

        ir::Function {
            ret_type: ir::Type::from_ast(&fun_def.ret_type.inner),
            name: fun_def.name.inner.clone(),
            args: ir_args,
            blocks: self.blocks,
        }
    }

    fn process_block<'b>(&mut self, block: &ast::Block, parent_label: ir::Label) -> ir::Label {
        let cur_label = self.alocate_new_block();
        self.env.allocate_new_frame(cur_label, parent_label);
        if parent_label != ARGS_LABEL {
            self.get_block(parent_label)
                .body
                .push(ir::Operation::Branch1(cur_label));
        }

        for stmt in &block.stmts {
            use model::ast::InnerStmt::*;
            match &stmt.inner {
                Empty => (),
                // Block(bl) => {
                //     self.process_block(bl, &cur_env);
                // }
                // Cond {
                //     cond,
                //     true_branch,
                //     false_branch,
                // } => {
                //     //todo
                // }
                // todo while
                // todo expr
                _ => unimplemented!(), // todo
            }
        }

        cur_label
    }

    // fn process_expression_cond(
    //     &mut self,
    //     expr: &ast::Expr,
    //     ops: &mut Vec<ir::Operation>,
    //     cur_env: &Env,
    // ) -> (ir::Label, ir::Label) {
    //     // todo
    //     // todo && and || and !
    //     unimplemented!()
    // }

    // fn process_expression(
    //     &mut self,
    //     expr: &ast::Expr,
    //     ops: &mut Vec<ir::Operation>,
    //     cur_env: &Env,
    // ) -> ir::Label {
    //     // todo
    //     // todo && and || and !
    //     unimplemented!()
    // }

    fn alocate_new_block(&mut self) -> ir::Label {
        let label = ir::Label(self.blocks.len() as u32);
        self.blocks.push(ir::Block {
            label,
            phi_set: HashSet::new(),
            body: vec![],
        });
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
