use std::collections::HashSet;

pub struct Program {
    pub structs: Vec<Struct>,
    pub functions: Vec<Function>,
    // todo (optional): global strings
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

pub type Label = u32;
pub type RegNum = u32;
pub struct Block {
    pub label: Label,
    pub phi_set: HashSet<(RegNum, Type, Vec<(Value, Label)>)>,
    pub body: Vec<Operation>,
    // todo should we remember here the following? probably no
    // predecessors
    // successors
    // map (per block): VarName -> Register
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

pub enum Value {
    LitInt(i32),
    LitBool(bool),
    LitNullPtr,
    Register(Type, RegNum),
}

pub enum Type {
    Int,
    Bool,
    Char,
    Ptr(Box<Type>),
    Struct(String),
}
