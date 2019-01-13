use model::{ast, ir};
use std::collections::HashMap;

// will take more arguments, probably
pub fn get_size_of_primitive(type_: &ir::Type) -> i32 {
    use self::ir::Type::*;
    match type_ {
        Void => unreachable!(),
        Int => 4,
        Bool => 1,
        Char => 1,
        Ptr(_) => 8, // 64-bit
        Class(_) => unreachable!(),
        Func(_, _) => unreachable!(),
    }
}

pub struct ClassRegistry<'a> {
    classes: HashMap<&'a str, ClassDescription<'a>>,
}

pub struct ClassDescription<'a> {
    fields: HashMap<&'a str, usize>,
    methods: HashMap<&'a str, usize>,
    class: ir::Class,
}

impl<'a> ClassRegistry<'a> {
    pub fn new() -> ClassRegistry<'a> {
        ClassRegistry {
            classes: HashMap::new(),
        }
    }

    pub fn process_class_def(&mut self, cl: &'a ast::ClassDef) {
        let mut cl_desc = if let Some(cl_type) = &cl.parent_type {
            match &cl_type.inner {
                ast::InnerType::Class(parent_cl_name) => ClassDescription::new_subclass(
                    &cl.name.inner,
                    &self.classes[parent_cl_name.as_str()],
                ),
                _ => unreachable!(),
            }
        } else {
            ClassDescription::new(&cl.name.inner)
        };

        let vtable_type = ir::get_class_vtable_type(&cl.name.inner);
        if cl_desc.class.fields.is_empty() {
            cl_desc.class.fields.push(vtable_type);
        } else {
            cl_desc.class.fields[0] = vtable_type;
        }

        for def in &cl.items {
            match &def.inner {
                ast::InnerClassItemDef::Field(f_type, f_name) => {
                    let ir_type = ir::Type::from_ast(&f_type.inner);
                    let new_idx = cl_desc.class.fields.len();
                    cl_desc.class.fields.push(ir_type);
                    cl_desc.fields.insert(&f_name.inner, new_idx);
                }
                ast::InnerClassItemDef::Method(fun) => {
                    let fun_type = ir::Type::from_method_def(&cl.name.inner, &fun);
                    let fun_name = ir::format_method_name(&cl.name.inner, &fun.name.inner);

                    // cloned to satisfy borrow checker
                    match cl_desc.methods.get(fun.name.inner.as_str()).cloned() {
                        Some(idx) => cl_desc.class.vtable[idx] = (fun_type, fun_name),
                        None => {
                            let new_idx = cl_desc.class.vtable.len();
                            cl_desc.class.vtable.push((fun_type, fun_name));
                            cl_desc.methods.insert(&fun.name.inner, new_idx);
                        }
                    }
                }
                ast::InnerClassItemDef::Error => unreachable!(),
            }
        }

        self.classes.insert(&cl.name.inner, cl_desc);
    }

    pub fn insert_classes_ir_into(self, program: &mut ir::Program) {
        for (_, cl) in self.classes.into_iter() {
            program.classes.push(cl.get_class_ir())
        }
    }
}

impl<'a> ClassDescription<'a> {
    fn new(name: &str) -> ClassDescription {
        ClassDescription {
            fields: HashMap::new(),
            methods: HashMap::new(),
            class: ir::Class {
                name: name.to_string(),
                fields: vec![],
                vtable: vec![],
            },
        }
    }

    fn new_subclass(name: &str, parent_cl_desc: &ClassDescription<'a>) -> ClassDescription<'a> {
        ClassDescription {
            fields: parent_cl_desc.fields.clone(),
            methods: parent_cl_desc.methods.clone(),
            class: ir::Class {
                name: name.to_string(),
                fields: parent_cl_desc.class.fields.clone(),
                vtable: parent_cl_desc.class.vtable.clone(),
            },
        }
    }

    fn get_class_ir(self) -> ir::Class {
        self.class
    }
}
