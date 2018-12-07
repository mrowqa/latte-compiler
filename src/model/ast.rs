#[derive(Debug)]
pub struct Program {
    pub defs: Vec<TopDef>,
}

#[derive(Debug)]
pub enum TopDef {
    FunDef(FunDef),
    ClassDef(ClassDef),
    Error,
}

pub type Span = (usize, usize);
pub type Ident = ItemWithSpan<String>;

#[derive(Debug)]
pub struct ClassDef {
    pub name: Ident,
    pub parent_name: Option<Ident>,
    pub items: Vec<ClassItemDef>,
    pub span: Span,
}

pub type ClassItemDef = ItemWithSpan<InnerClassItemDef>;
#[derive(Debug)]
pub enum InnerClassItemDef {
    Field(Type, Ident),
    Method(FunDef),
    Error,
}

#[derive(Debug)]
pub struct FunDef {
    pub ret_type: Type,
    pub name: Ident,
    pub args: Vec<(Type, Ident)>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Box<Stmt>>,
    pub span: Span,
}

#[derive(Debug)]
pub struct ItemWithSpan<T> { // todo (rename to Node<T>)
    pub inner: T,
    pub span: Span,
}

// global functions, because it's shorter to write in grammar file
pub fn new_spanned_boxed<T>(l: usize, inner: T, r: usize) -> Box<ItemWithSpan<T>> {
    Box::new(new_spanned(l, inner, r))
}
pub fn new_spanned<T>(l: usize, inner: T, r: usize) -> ItemWithSpan<T> {
    ItemWithSpan {
        inner: inner,
        span: (l, r),
    }
}

pub type Stmt = ItemWithSpan<InnerStmt>;
#[derive(Debug)]
pub enum InnerStmt {
    Empty,
    Block(Block),
    Decl{var_type: Type, var_items: Vec<(Ident, Option<Box<Expr>>)>},
    Assign(Box<Expr>, Box<Expr>),
    Incr(Box<Expr>),
    Decr(Box<Expr>),
    Ret(Option<Box<Expr>>),
    Cond{cond: Box<Expr>, true_branch: Box<Stmt>, false_branch: Option<Box<Stmt>>},
    While(Box<Expr>, Box<Stmt>),
    ForEach{iter_type: Type, iter_name: Ident, array: Box<Expr>, body: Box<Stmt>},
    Expr(Box<Expr>),
    Error,
}

pub type Type = ItemWithSpan<InnerType>;
#[derive(Debug)]
pub enum InnerType {
    Int,
    Bool,
    String,
    Array(Box<Type>), // todo leave only innertype?
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
    ObjMethodCall{obj: Box<Expr>, method_name: Ident, args: Vec<Box<Expr>>},
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
