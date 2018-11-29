pub struct Program {
    functions: Vec<Function>,
    classes: Vec<Class>,
}

type Span = (usize, usize);

pub struct Class {
    name: String,
    parent_name: Option<String>,
    fields: Vec<(Type, String)>,
    methods: Vec<Function>,
    span: Span,
}

pub struct Function {
    ret_type: Type,
    name: String,
    args: Vec<(Type, String)>,
    body: Stmt,
    span: Span,
}

pub struct Block {
    stmts: Vec<Stmt>,
    span: Span,
}

pub struct ItemWithSpan<T> {
    inner: T,
    span: Span,
}

pub type Stmt = ItemWithSpan<InnerStmt>;
pub enum InnerStmt {
    Empty,
    Block(Block),
    Decl{var_type: Type, var_items: Vec<(String, Option<Expr>)>},
    Assign(Expr, Expr),
    Ret(Option<Expr>),
    Cond{cond: Expr, true_branch: Box<Stmt>, false_branch: Option<Box<Stmt>>},
    While(Expr, Box<Stmt>),
    ForEach{iter_type: Type, iter_name: String, array: Expr},
    Expr(Expr),
}

pub type Type = ItemWithSpan<InnerType>;
pub enum InnerType {
    Int,
    Bool,
    String,
    Array(Box<Type>),
    Class(String),
    Void,
}

pub type Expr = ItemWithSpan<InnerExpr>;
pub enum InnerExpr {
    LitVar(String),
    LitInt(i32),
    LitBool(bool),
    LitStr(String),
    LitNull,
    FunCall{function_name: String, args: Vec<Expr>},
    BinaryOp(Box<Expr>, BinaryOp, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
    NewArray{elem_type: Type, elem_cnt: Box<Expr>},
    ArrayElem{array: Box<Expr>, index: Box<Expr>},
    NewObject(Type),
    ObjField{obj: Box<Expr>, field: String},
    ObjMethodCall{obj: Box<Expr>, method_name: String, args: Vec<Expr>},
}

pub type UnaryOp = ItemWithSpan<UnaryOpInner>;
pub enum UnaryOpInner {
    IntNeg,
    BoolNeg,
}

pub enum BinaryOp {
    And,
    Or,
    Plus,
    Minus,
    Mult,
    Div,
    Mod,
    LT,
    LE,
    GT,
    GE,
    EQ,
    NE,
}
