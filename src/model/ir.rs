use model::ast;
use std::collections::HashSet;

pub struct Program {
    pub structs: Vec<Struct>,
    pub functions: Vec<Function>,
    // todo: global strings
}

pub struct Struct {
    pub name: String,
    pub fields: Vec<Type>,
}

pub struct Function {
    pub ret_type: Type,
    pub name: String,
    pub args: Vec<(Type, String)>,
    pub blocks: Vec<Block>,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Label(pub u32);

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct RegNum(pub u32);

pub struct Block {
    pub label: Label,
    pub phi_set: HashSet<(RegNum, Type, Vec<(Value, Label)>)>,
    pub body: Vec<Operation>,
}

// almost-quadruple code
// read left-to-right, like in LLVM
pub enum Operation {
    Return(Option<Value>),
    FunctionCall(Option<RegNum>, Type, String, Vec<Value>),
    Arithmetic(RegNum, ArithOp, Value, Value),
    Compare(RegNum, CmpOp, Value, Value),
    GetElementPtr(RegNum, Type, Value, Value),
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

#[derive(PartialEq, Eq, Hash)]
pub enum Value {
    LitInt(i32),
    LitBool(bool),
    LitNullPtr,
    Register(Type, RegNum),
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

impl Type {
    pub fn from_ast(ast_type: &ast::InnerType) -> Type {
        match ast_type {
            ast::InnerType::Int => Type::Int,
            ast::InnerType::Bool => Type::Bool,
            ast::InnerType::String => Type::Ptr(Box::new(Type::Char)),
            ast::InnerType::Array(subtype) => Type::Ptr(Box::new(Type::from_ast(&subtype))),
            ast::InnerType::Class(name) => Type::Ptr(Box::new(Type::Struct(name.clone()))),
            ast::InnerType::Null => Type::Ptr(Box::new(Type::Void)),
            ast::InnerType::Void => Type::Void,
        }
    }
}
// todo?
// value : to type
