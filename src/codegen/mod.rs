use codegen::{class::ClassRegistry, function::FunctionCodeGen};
use model::{ast, ir};
use semantics::global_context::GlobalContext;
use std::collections::{HashMap, VecDeque};

mod class;
mod function;

pub struct CodeGen<'a> {
    ast: &'a ast::Program,
    gctx: &'a GlobalContext<'a>,
}

impl<'a> CodeGen<'a> {
    pub fn new(ast: &'a ast::Program, gctx: &'a GlobalContext<'a>) -> CodeGen<'a> {
        CodeGen { ast, gctx }
    }

    pub fn generate_ir(&self) -> ir::Program {
        let mut prog_ir = ir::Program {
            classes: vec![],
            functions: vec![],
            global_strings: HashMap::new(),
        };
        let mut class_registry = ClassRegistry::new();

        self.calculate_class_registry(&mut class_registry);
        self.generate_functions_ir(&mut prog_ir, &class_registry);
        class_registry.insert_classes_ir_into(&mut prog_ir);

        prog_ir
    }

    fn calculate_class_registry(&self, class_registry: &mut ClassRegistry<'a>) {
        let mut class_queue = VecDeque::new();
        let mut class_hierarchy = HashMap::new();
        for def in &self.ast.defs {
            if let ast::TopDef::ClassDef(cl) = def {
                match &cl.parent_type {
                    Some(ast::ItemWithSpan {
                        inner: ast::InnerType::Class(parent_name),
                        ..
                    }) => {
                        class_hierarchy
                            .entry(parent_name)
                            .or_insert(vec![])
                            .push(cl);
                    }
                    None => {
                        class_registry.process_class_def(&cl);
                        class_queue.push_back(&cl.name.inner);
                    }
                    _ => unreachable!(),
                }
            }
        }
        while let Some(cl_name) = class_queue.pop_front() {
            if let Some(sons) = class_hierarchy.get(&cl_name) {
                for cl in sons {
                    class_registry.process_class_def(&cl);
                    class_queue.push_back(&cl.name.inner);
                }
            }
        }
    }

    fn generate_functions_ir(&self, prog_ir: &mut ir::Program, class_registry: &ClassRegistry) {
        for def in &self.ast.defs {
            match def {
                ast::TopDef::FunDef(fun) => {
                    let fun_cg = FunctionCodeGen::new(
                        &self.gctx,
                        None,
                        &mut prog_ir.global_strings,
                        &class_registry,
                    );
                    let fun_ir = fun_cg.generate_function_ir(&fun);
                    prog_ir.functions.push(fun_ir);
                }
                ast::TopDef::ClassDef(cl) => {
                    let cl_desc = self.gctx.get_class_description(&cl.name.inner).unwrap();
                    for it in &cl.items {
                        match &it.inner {
                            ast::InnerClassItemDef::Field(_, _) => (),
                            ast::InnerClassItemDef::Method(fun) => {
                                let fun_cg = FunctionCodeGen::new(
                                    &self.gctx,
                                    Some(cl_desc),
                                    &mut prog_ir.global_strings,
                                    &class_registry,
                                );
                                let fun_ir = fun_cg.generate_function_ir(&fun);
                                prog_ir.functions.push(fun_ir);
                            }
                            ast::InnerClassItemDef::Error => unreachable!(),
                        }
                    }
                }
                ast::TopDef::Error => unreachable!(),
            }
        }
    }
}
