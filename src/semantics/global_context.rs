use std::collections::HashMap;
use model::ast::*;
use frontend_error::{FrontendError, FrontendResult, ErrorAccumulation};

pub struct GlobalContext<'a> {
    classes: HashMap<&'a str, ClassDesc<'a>>,
    functions: HashMap<&'a str, FunDesc<'a>>,
}

struct ClassDesc<'a> {
    name: &'a str,
    #[allow(dead_code)] // todo remove
    parent_name: Option<&'a str>,
    fields: HashMap<&'a str, &'a Type>,
    methods: HashMap<&'a str, FunDesc<'a>>,
}

struct FunDesc<'a> {
    pub ret_type: &'a Type,
    pub name: &'a str,
    pub args_types: Vec<&'a Type>,
}

impl<'a> GlobalContext<'a> {
    fn new_with_builtins() -> Self {
        // todo add builtins
        GlobalContext {
            classes: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn from(prog: &'a Program) -> FrontendResult<Self> {
        let mut result = GlobalContext::new_with_builtins();
        let mut errors = vec![];
        result.scan_global_defenitions(prog).accumulate_errors_in(&mut errors);
        result.check_types_in_defs().accumulate_errors_in(&mut errors);

        if errors.is_empty() { Ok(result) } else { Err(errors) }
    }

    fn scan_global_defenitions(&mut self, prog: &'a Program) -> FrontendResult<()> {
        let mut errors = vec![];
        for def in &prog.defs {
            match def {
                TopDef::FunDef(fun) => {
                    let fun_desc = FunDesc::from(&fun);
                    if let Some(_) = self.functions.insert(fun_desc.name, fun_desc) {
                        errors.push(FrontendError {
                            err: "Error: function redefinition".to_string(),
                            span: fun.name.span,
                         });
                    }
                },
                TopDef::ClassDef(cl) => {
                    let class_desc_res = ClassDesc::from(&cl);
                    match class_desc_res {
                        Ok(desc) => {
                            if let Some(_) = self.classes.insert(desc.name, desc) {
                                errors.push(FrontendError {
                                    err: "Error: class redefinition".to_string(),
                                    span: cl.name.span,
                                });
                            }
                        },
                        Err(err) => errors.extend(err),
                    }
                },
                TopDef::Error => unreachable!(),
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    fn check_types_in_defs(&mut self) -> FrontendResult<()> {
        let mut errors = vec![];
        for f in self.functions.values() {
            f.check_types(&self).accumulate_errors_in(&mut errors);
        }
        for c in self.classes.values() {
            c.check_types(&self).accumulate_errors_in(&mut errors);
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    pub fn check_type(&self, t: &Type) -> FrontendResult<()> {
        use self::InnerType::*;
        match &t.inner {
            Array(subtype) => self.check_type(&subtype), // theoretically we could pass the span, so it would contain trailing "[]"
            Class(name) => {
                if self.classes.contains_key(name.as_str()) {
                    Ok(())
                }
                else {
                    Err(vec![FrontendError {
                        err: "Error: invalid type - class not defined".to_string(),
                        span: t.span,
                    }])
                }
            },
            Void => Err(vec![FrontendError {
                err: "Error: invalid type - cannot use void here".to_string(),
                span: t.span,
            }]),
            Int | Bool | String => Ok(()),
        }
    }

    pub fn check_ret_type(&self, t: &Type) -> FrontendResult<()> {
        if let InnerType::Void = t.inner {
            Ok(())
        }
        else {
            self.check_type(t)
        }
    }
}

impl<'a> ClassDesc<'a> {
    pub fn from(cldef: &'a ClassDef) -> FrontendResult<Self> {
        let mut errors = vec![];
        let mut result = ClassDesc {
            name: &cldef.name.inner,
            parent_name: cldef.parent_name.as_ref().map(|id| id.inner.as_str()),
            fields: HashMap::new(),
            methods: HashMap::new(),
        };

        for item in &cldef.items {
            match &item.inner {
                InnerClassItemDef::Field(t, id) => {
                    if let Some(_) = result.fields.insert(&id.inner, t) {
                        errors.push(FrontendError {
                            err: "Error: field redefinition".to_string(),
                            span: item.span,
                         });
                    }
                },
                InnerClassItemDef::Method(fun) => {
                    let fun_desc = FunDesc::from(&fun);
                    if let Some(_) = result.methods.insert(fun_desc.name, fun_desc) {
                        errors.push(FrontendError {
                            err: "Error: method redefinition".to_string(),
                            span: fun.name.span,
                         });
                    }
                },
                InnerClassItemDef::Error => unreachable!(),
            }
        }

        if errors.is_empty() { Ok(result) } else { Err(errors) }
    }

    pub fn check_types(&self, ctx: &GlobalContext<'a>) -> FrontendResult<()> {
        let mut errors = vec![];
        for t in self.fields.values() {
            ctx.check_type(t).accumulate_errors_in(&mut errors);
        }
        for f in self.methods.values() {
            f.check_types(ctx).accumulate_errors_in(&mut errors);
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
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
        ctx.check_ret_type(self.ret_type).accumulate_errors_in(&mut errors);
        for t in &self.args_types {
            ctx.check_type(t).accumulate_errors_in(&mut errors);
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
