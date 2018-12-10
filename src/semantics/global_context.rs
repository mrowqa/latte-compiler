use frontend_error::{ok_if_no_error, ErrorAccumulation, FrontendError, FrontendResult};
use model::ast::*;
use std::collections::HashMap;

pub struct GlobalContext<'a> {
    classes: HashMap<&'a str, ClassDesc<'a>>,
    functions: HashMap<&'a str, FunDesc<'a>>,
}

pub struct ClassDesc<'a> {
    name: &'a str,
    parent_type: Option<&'a Type>,
    items: HashMap<&'a str, TypeWrapper<'a>>,
}

pub enum TypeWrapper<'a> {
    Var(&'a Type),
    Fun(FunDesc<'a>),
}

pub struct FunDesc<'a> {
    // todo (optional) use getters instead of pub fields?
    pub ret_type: &'a Type,
    pub name: &'a str,
    pub args_types: Vec<&'a Type>,
}

impl<'a> GlobalContext<'a> {
    fn new_with_builtins() -> Self {
        GlobalContext {
            classes: HashMap::new(),
            functions: get_builtin_functions(),
        }
    }

    pub fn from(prog: &'a Program) -> FrontendResult<Self> {
        let mut result = GlobalContext::new_with_builtins();
        let mut errors = vec![];
        result
            .scan_global_defenitions(prog)
            .accumulate_errors_in(&mut errors);
        result
            .check_types_in_context_defs()
            .accumulate_errors_in(&mut errors);

        if errors.is_empty() {
            Ok(result)
        } else {
            Err(errors)
        }
    }

    pub fn get_class_description(&self, cl_name: &str) -> Option<&ClassDesc<'a>> {
        self.classes.get(cl_name)
    }

    pub fn get_function_description(&self, fun_name: &str) -> Option<&FunDesc<'a>> {
        self.functions.get(fun_name)
    }

    fn scan_global_defenitions(&mut self, prog: &'a Program) -> FrontendResult<()> {
        let mut errors = vec![];
        for def in &prog.defs {
            match def {
                TopDef::FunDef(fun) => {
                    let fun_desc = FunDesc::from(&fun);
                    if self.functions.insert(fun_desc.name, fun_desc).is_some() {
                        errors.push(FrontendError {
                            err: "Error: function redefinition".to_string(),
                            span: fun.name.span,
                        });
                    }
                }
                TopDef::ClassDef(cl) => {
                    let class_desc_res = ClassDesc::from(&cl);
                    match class_desc_res {
                        Ok(desc) => {
                            if self.classes.insert(desc.name, desc).is_some() {
                                errors.push(FrontendError {
                                    err: "Error: class redefinition".to_string(),
                                    span: cl.name.span,
                                });
                            }
                        }
                        Err(err) => errors.extend(err),
                    }
                }
                TopDef::Error => unreachable!(),
            }
        }

        ok_if_no_error(errors)
    }

    fn check_types_in_context_defs(&mut self) -> FrontendResult<()> {
        let mut errors = vec![];
        for f in self.functions.values() {
            f.check_types(&self).accumulate_errors_in(&mut errors);
        }
        for c in self.classes.values() {
            c.check_types(&self).accumulate_errors_in(&mut errors);
        }

        ok_if_no_error(errors)
    }

    pub fn check_local_var_type(&self, t: &Type) -> FrontendResult<()> {
        use self::InnerType::*;
        match &t.inner {
            Array(subtype) => {
                let tt = Type {
                    inner: *subtype.clone(),
                    span: t.span,
                };
                self.check_local_var_type(&tt)
            }
            Class(name) => {
                if self.classes.contains_key(name.as_str()) {
                    Ok(())
                } else {
                    Err(vec![FrontendError {
                        err: "Error: invalid type - class not defined".to_string(),
                        span: t.span,
                    }])
                }
            }
            Void => Err(vec![FrontendError {
                err: "Error: invalid type - cannot use void here".to_string(),
                span: t.span,
            }]),
            Int | Bool | String => Ok(()),
            Null => unreachable!(),
        }
    }

    pub fn check_ret_type(&self, t: &Type) -> FrontendResult<()> {
        if let InnerType::Void = t.inner {
            Ok(())
        } else {
            self.check_local_var_type(t)
        }
    }

    pub fn check_superclass_type(&self, t: &Type, my_name: &str) -> FrontendResult<()> {
        if let InnerType::Class(parent_name) = &t.inner {
            self.check_for_inheritance_cycle(my_name, &parent_name, t.span)
        } else {
            Err(vec![FrontendError {
                err: "Error: super class must be a class".to_string(),
                span: t.span,
            }])
        }
    }

    fn check_for_inheritance_cycle(
        &self,
        start_name: &str,
        cur_name: &str,
        span: Span,
    ) -> FrontendResult<()> {
        if let Some(cl) = self.classes.get(cur_name) {
            if cl.name == start_name {
                Err(vec![FrontendError {
                    err: "Error: detected cycle in inheritance chain".to_string(),
                    span: span,
                }])
            } else if let Some(t) = cl.parent_type {
                match &t.inner {
                    InnerType::Class(parent_name) => {
                        self.check_for_inheritance_cycle(start_name, &parent_name, span)
                    }
                    _ => unreachable!(), // assumption: tree made by our parser
                }
            } else {
                Ok(())
            }
        } else {
            Err(vec![FrontendError {
                err: "Error: invalid type - class not defined".to_string(),
                span: span,
            }])
        }
    }

    pub fn check_types_compatibility(
        &self,
        lhs: &'a InnerType,
        rhs: &'a InnerType,
        span: Span,
    ) -> FrontendResult<()> {
        // todo (ext) allow the following:
        // lhs=array/obj, rhs=null
        // lhs=superclass, rhs=subclass
        if lhs == rhs {
            Ok(())
        } else {
            let err = format!("Error: expected type {}, got type {}", lhs, rhs);
            Err(vec![FrontendError { err, span }])
        }
    }
}

impl<'a> ClassDesc<'a> {
    pub fn from(cldef: &'a ClassDef) -> FrontendResult<Self> {
        let mut errors = vec![];
        let mut result = ClassDesc {
            name: &cldef.name.inner,
            parent_type: cldef.parent_type.as_ref(),
            items: HashMap::new(),
        };

        // scope for the closure which borrows errors
        {
            let mut add_or_error = |name: &'a str, t: TypeWrapper<'a>, span: Span| {
                if result.items.insert(name, t).is_some() {
                    errors.push(FrontendError {
                        err: "Error: class item redefinition".to_string(),
                        span: span,
                    });
                }
            };

            for item in &cldef.items {
                match &item.inner {
                    InnerClassItemDef::Field(t, id) => {
                        add_or_error(&id.inner, TypeWrapper::Var(t), item.span)
                    }
                    InnerClassItemDef::Method(fun) => {
                        let fun_desc = FunDesc::from(&fun);
                        add_or_error(fun_desc.name, TypeWrapper::Fun(fun_desc), fun.name.span)
                    }
                    InnerClassItemDef::Error => unreachable!(),
                }
            }
        }

        if errors.is_empty() {
            Ok(result)
        } else {
            Err(errors)
        }
    }

    pub fn check_types(&self, ctx: &GlobalContext<'a>) -> FrontendResult<()> {
        // todo (ext) check if proper method signature when overriding method
        // todo (ext) some helper method for looking up methods in superclasses
        // todo (ext) handle properly fields vs methods while inheriting them
        let mut errors = vec![];
        if let Some(t) = self.parent_type {
            ctx.check_superclass_type(t, self.name)
                .accumulate_errors_in(&mut errors);
        }
        for t in self.items.values() {
            match t {
                TypeWrapper::Var(var_type) => ctx
                    .check_local_var_type(var_type)
                    .accumulate_errors_in(&mut errors),
                TypeWrapper::Fun(fun_desc) => {
                    fun_desc.check_types(ctx).accumulate_errors_in(&mut errors)
                }
            }
        }

        ok_if_no_error(errors)
    }

    pub fn get_item(&self, name: &str) -> Option<&TypeWrapper<'a>> {
        self.items.get(name)
    }
}

impl<'a> FunDesc<'a> {
    pub fn from(fundef: &'a FunDef) -> Self {
        FunDesc {
            ret_type: &fundef.ret_type,
            name: &fundef.name.inner,
            args_types: fundef.args.iter().map(|(t, _)| t).collect(),
        }
    }

    pub fn check_types(&self, ctx: &GlobalContext<'a>) -> FrontendResult<()> {
        let mut errors = vec![];
        ctx.check_ret_type(self.ret_type)
            .accumulate_errors_in(&mut errors);
        for t in &self.args_types {
            ctx.check_local_var_type(t)
                .accumulate_errors_in(&mut errors);
        }

        ok_if_no_error(errors)
    }
}

// --------------------------------------------------------
// ----------------- builtins -----------------------------
// --------------------------------------------------------
fn get_builtin_functions() -> HashMap<&'static str, FunDesc<'static>> {
    let t_void = &Type {
        inner: InnerType::Void,
        span: (0, 0),
    };
    let t_int = &Type {
        inner: InnerType::Int,
        span: (0, 0),
    };
    let t_string = &Type {
        inner: InnerType::String,
        span: (0, 0),
    };

    let mut m = HashMap::new();
    m.insert(
        "printInt",
        FunDesc {
            ret_type: t_void,
            name: "printInt",
            args_types: vec![t_int],
        },
    );
    m.insert(
        "printString",
        FunDesc {
            ret_type: t_void,
            name: "printString",
            args_types: vec![t_string],
        },
    );
    m.insert(
        "error",
        FunDesc {
            ret_type: t_void,
            name: "error",
            args_types: vec![],
        },
    );
    m.insert(
        "readInt",
        FunDesc {
            ret_type: t_int,
            name: "readInt",
            args_types: vec![],
        },
    );
    m.insert(
        "readString",
        FunDesc {
            ret_type: t_string,
            name: "readString",
            args_types: vec![],
        },
    );
    m
}
