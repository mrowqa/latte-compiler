#[derive(Debug)]
pub struct Program {
    functions: Vec<Function>,
    classes: Vec<Class>,
}

type Span = (usize, usize);
pub type Ident = ItemWithSpan<String>;

#[derive(Debug)]
pub struct Class {
    name: Ident,
    parent_name: Option<Ident>,
    fields: Vec<(Type, Ident)>,
    methods: Vec<Function>,
    span: Span,
}

#[derive(Debug)]
pub struct Function {
    ret_type: Type,
    name: Ident,
    args: Vec<(Type, Ident)>,
    body: Stmt,
    span: Span,
}

#[derive(Debug)]
pub struct Block {
    stmts: Vec<Stmt>,
    span: Span,
}

#[derive(Debug)]
pub struct ItemWithSpan<T> {
    pub inner: T,
    pub span: Span,
}

// global function, because it's shorter to write in grammar file
pub fn new_spanned_boxed<T>(l: usize, inner: T, r: usize) -> Box<ItemWithSpan<T>> {
    Box::new(ItemWithSpan {
        inner: inner,
        span: (l, r),
    })
}

pub type Stmt = ItemWithSpan<InnerStmt>;
#[derive(Debug)]
pub enum InnerStmt {
    Empty,
    Block(Block),
    Decl{var_type: Type, var_items: Vec<(Ident, Option<Expr>)>},
    Assign(Expr, Expr),
    Ret(Option<Expr>),
    Cond{cond: Expr, true_branch: Box<Stmt>, false_branch: Option<Box<Stmt>>},
    While(Expr, Box<Stmt>),
    ForEach{iter_type: Type, iter_name: Ident, array: Expr},
    Expr(Expr),
}

pub type Type = ItemWithSpan<InnerType>;
#[derive(Debug)]
pub enum InnerType {
    Int,
    Bool,
    String,
    Array(Box<Type>),
    Class(String),
    Void,
}

pub type Expr = ItemWithSpan<InnerExpr>;
#[derive(Debug)]
pub enum InnerExpr {
    LitVar(String),
    LitInt(i32),
    LitBool(bool),
    LitStr(String),
    LitNull,
    FunCall{function_name: Ident, args: Vec<Box<Expr>>},
    BinaryOp(Box<Expr>, BinaryOp, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
    NewArray{elem_type: Type, elem_cnt: Box<Expr>},
    ArrayElem{array: Box<Expr>, index: Box<Expr>},
    NewObject(Type),
    ObjField{obj: Box<Expr>, field: Ident},
    ObjMethodCall{obj: Box<Expr>, method_name: Ident, args: Vec<Expr>},
}

pub type UnaryOp = ItemWithSpan<UnaryOpInner>;
#[derive(Debug)]
pub enum UnaryOpInner {
    IntNeg,
    BoolNeg,
}

#[derive(Debug)]
pub enum BinaryOp {
    And,
    Or,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    LT,
    LE,
    GT,
    GE,
    EQ,
    NE,
}
