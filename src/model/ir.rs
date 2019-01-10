use model::ast;
use std::collections::{HashMap, HashSet};
use std::fmt;

pub struct Program {
    pub structs: Vec<Struct>,
    pub functions: Vec<Function>,
    pub global_strings: HashMap<String, GlobalStrNum>,
}

pub struct Struct {
    pub name: String,
    pub fields: Vec<Type>,
}

pub struct Function {
    pub ret_type: Type,
    pub name: String,
    pub args: Vec<(RegNum, Type)>,
    pub blocks: Vec<Block>,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Label(pub u32);

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct RegNum(pub u32);

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct GlobalStrNum(pub u32);

pub struct Block {
    pub label: Label,
    pub phi_set: HashSet<PhiEntry>,
    pub predecessors: Vec<Label>,
    pub body: Vec<Operation>,
}
pub type PhiEntry = (RegNum, Type, Vec<(Value, Label)>); // todo (optional) add string for var name

// almost-quadruple code
// read left-to-right, like in LLVM
pub enum Operation {
    Return(Option<Value>),
    FunctionCall(Option<RegNum>, Type, String, Vec<Value>),
    Arithmetic(RegNum, ArithOp, Value, Value),
    Compare(RegNum, CmpOp, Value, Value),
    GetElementPtr(RegNum, Type, Value, Value),
    CastGlobalString(RegNum, usize, GlobalStrNum), // usize is string length
    CastPtr {
        dst: RegNum,
        dst_type: Type,
        src_value: Value,
    },
    Load(RegNum, Value),
    Store(Value, Value),
    Branch1(Label),
    Branch2(Value, Label, Label),
}

pub enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

pub enum CmpOp {
    LT,
    LE,
    GT,
    GE,
    EQ,
    NE,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Value {
    LitInt(i32),
    LitBool(bool),
    LitNullPtr(Option<Type>),
    Register(RegNum, Type),
    GlobalRegister(GlobalStrNum),
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Type {
    Void,
    Int,
    Bool,
    Char,
    Ptr(Box<Type>),
    Struct(String),
}

impl Value {
    pub fn get_type(&self) -> Type {
        match self {
            Value::LitInt(_) => Type::Int,
            Value::LitBool(_) => Type::Bool,
            Value::LitNullPtr(Some(t)) => t.clone(),
            Value::LitNullPtr(None) => Type::Ptr(Box::new(Type::Char)), // void* is illegal in llvm
            Value::Register(_, t) => t.clone(),
            Value::GlobalRegister(_) => Type::Ptr(Box::new(Type::Char)),
        }
    }
}

impl Type {
    pub fn from_ast(ast_type: &ast::InnerType) -> Type {
        match ast_type {
            ast::InnerType::Int => Type::Int,
            ast::InnerType::Bool => Type::Bool,
            ast::InnerType::String => Type::Ptr(Box::new(Type::Char)),
            ast::InnerType::Array(subtype) => Type::Ptr(Box::new(Type::from_ast(&subtype))),
            ast::InnerType::Class(name) => Type::Ptr(Box::new(Type::Struct(name.clone()))),
            ast::InnerType::Null => Type::Ptr(Box::new(Type::Char)),
            ast::InnerType::Void => Type::Void,
        }
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            r#"declare void @printInt(i32)
declare void @printString(i8*)
declare void @error()
declare i32  @readInt()
declare i8*  @readString()
declare i8*  @_bltn_string_concat(i8*, i8*)
declare i1   @_bltn_string_eq(i8*, i8*)
declare i1   @_bltn_string_ne(i8*, i8*)
declare i8*  @_bltn_malloc(i32)
declare i8*  @_bltn_alloc_array(i32, i32)

"#
        )?;

        for (k, v) in self.global_strings.iter() {
            writeln!(
                f,
                r#"@.str.{} = private constant [{} x i8] c"{}\00""#,
                v.0,
                k.len() + 1,
                k.replace("\\", "\\5C")
                    .replace("\"", "\\22")
                    .replace("\n", "\\0A")
                    .replace("\t", "\\09")
            )?;
        }
        write!(f, "\n\n")?;

        // todo (ext) structs

        for fun in &self.functions {
            fun.fmt(f)?;
        }

        Ok(())
    }
}

#[allow(dead_code)] // todo (ext) remove
impl fmt::Display for Struct {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!() // todo (ext)
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let priv_str = if self.name == "main" { "" } else { "private " };
        write!(f, "define {}{} @{}(", priv_str, self.ret_type, self.name)?;
        for (i, (reg_num, arg_type)) in self.args.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{} %.r{}", arg_type, reg_num.0)?;
        }
        writeln!(f, ") {{")?;

        for bl in &self.blocks {
            bl.fmt(f)?;
        }
        write!(f, "}}\n\n")
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, ".L{}:", self.label.0)?;
        if !self.predecessors.is_empty() {
            write!(f, "  ; preds: ")?;
            for (i, pred_label) in self.predecessors.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "%.L{}", pred_label.0)?;
            }
        }
        writeln!(f)?;

        for (reg_num, reg_type, vals) in &self.phi_set {
            write!(f, "    %.r{} = phi {} ", reg_num.0, reg_type)?;
            for (i, (value, label)) in vals.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "[{}, %.L{}]", value, label.0)?;
            }
            writeln!(f)?;
        }

        for op in &self.body {
            writeln!(f, "    {}", op)?;
        }

        Ok(())
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Operation::*;
        match self {
            Return(opt_val) => match opt_val {
                Some(val) => write!(f, "ret {} {}", val.get_type(), val)?,
                None => write!(f, "ret void")?,
            },
            FunctionCall(opt_reg_num, ret_type, fun_name, args) => {
                match opt_reg_num {
                    Some(reg_num) => write!(f, "%.r{} = ", reg_num.0)?,
                    None => (),
                }

                write!(f, "call {} @{}(", ret_type, fun_name)?;
                for (i, val) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{} {}", val.get_type(), val)?;
                }
                write!(f, ")")?;
            }
            Arithmetic(reg_num, op, val1, val2) => {
                use self::ArithOp::*;
                let op_str = match op {
                    Add => "add",
                    Sub => "sub",
                    Mul => "mul",
                    Div => "sdiv",
                    Mod => "srem",
                };
                write!(
                    f,
                    "%.r{} = {} {} {}, {}",
                    reg_num.0,
                    op_str,
                    val1.get_type(),
                    val1,
                    val2
                )?;
            }
            Compare(reg_num, op, val1, val2) => {
                use self::CmpOp::*;
                let op_str = match op {
                    LT => "slt",
                    LE => "sle",
                    GT => "sgt",
                    GE => "sge",
                    EQ => "eq",
                    NE => "ne",
                };
                let val_type = match val1 {
                    Value::LitNullPtr(_) => val2.get_type(),
                    _ => val1.get_type(),
                };
                write!(
                    f,
                    "%.r{} = icmp {} {} {}, {}",
                    reg_num.0, op_str, val_type, val1, val2
                )?;
            }
            GetElementPtr(reg_num, elem_type, ptr_val, ind_val) => {
                write!(
                    f,
                    "%.r{} = getelementptr {}, {} {}, {} {}",
                    reg_num.0,
                    elem_type,
                    ptr_val.get_type(),
                    ptr_val,
                    ind_val.get_type(),
                    ind_val,
                )?;
            }
            CastGlobalString(reg_num, str_len, str_num) => {
                write!(
                    f,
                    "%.r{0} = getelementptr [{1} x i8], [{1} x i8]* @.str.{2}, i32 0, i32 0",
                    reg_num.0, str_len, str_num.0,
                )?;
            }
            CastPtr {
                dst,
                dst_type,
                src_value,
            } => {
                let (val_reg, val_type) = match src_value {
                    Value::Register(val_reg, val_type) => (val_reg, val_type),
                    _ => unreachable!(),
                };
                write!(
                    f,
                    "%.r{} = bitcast {} %.r{} to {}",
                    dst.0, val_type, val_reg.0, dst_type
                )?;
            }
            Load(reg_num, value) => {
                let (val_reg, elem_type) = match value {
                    Value::Register(val_reg, Type::Ptr(subtype)) => (val_reg, subtype),
                    _ => unreachable!(),
                };
                write!(
                    f,
                    "%.r{0} = load {1}, {1}* %.r{2}",
                    reg_num.0, elem_type, val_reg.0
                )?;
            }
            Store(target_val, ref_val) => {
                write!(
                    f,
                    "store {} {}, {} {}",
                    target_val.get_type(),
                    target_val,
                    ref_val.get_type(),
                    ref_val
                )?;
            }
            Branch1(label) => {
                write!(f, "br label %.L{}", label.0)?;
            }
            Branch2(value, label1, label2) => {
                write!(
                    f,
                    "br i1 {}, label %.L{}, label %.L{}",
                    value, label1.0, label2.0
                )?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Value::*;
        match self {
            LitInt(val) => val.fmt(f),
            LitBool(val) => (*val as i32).fmt(f),
            LitNullPtr(_) => "null".fmt(f),
            Register(reg_num, _) => write!(f, "%.r{}", reg_num.0),
            GlobalRegister(str_num) => write!(f, "@.str.{}", str_num.0),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Type::*;
        match self {
            Void => write!(f, "void"),
            Int => write!(f, "i32"),
            Bool => write!(f, "i1"),
            Char => write!(f, "i8"),
            Ptr(subtype) => write!(f, "{}*", subtype),
            Struct(name) => write!(f, "%.struct.{}", name),
        }
    }
}
