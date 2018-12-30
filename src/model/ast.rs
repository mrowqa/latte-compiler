use std::fmt;

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
pub const EMPTY_SPAN: Span = (0, 0);
pub type Ident = ItemWithSpan<String>;

#[derive(Debug)]
pub struct ClassDef {
    pub name: Ident,
    pub parent_type: Option<Type>,
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
pub struct ItemWithSpan<T> {
    // todo (optional) rename to Node<T>
    pub inner: T,
    pub span: Span,
}

// global functions, because it's shorter to write in grammar file
pub fn new_spanned_boxed<T>(l: usize, inner: T, r: usize) -> Box<ItemWithSpan<T>> {
    Box::new(new_spanned(l, inner, r))
}
pub fn new_spanned<T>(l: usize, inner: T, r: usize) -> ItemWithSpan<T> {
    ItemWithSpan {
        inner,
        span: (l, r),
    }
}

pub type Stmt = ItemWithSpan<InnerStmt>;
#[derive(Debug)]
pub enum InnerStmt {
    Empty,
    Block(Block),
    Decl {
        var_type: Type,
        var_items: Vec<(Ident, Option<Box<Expr>>)>,
    },
    Assign(Box<Expr>, Box<Expr>),
    Incr(Box<Expr>),
    Decr(Box<Expr>),
    Ret(Option<Box<Expr>>),
    Cond {
        cond: Box<Expr>,
        true_branch: Block,
        false_branch: Option<Block>,
    },
    While(Box<Expr>, Block),
    ForEach {
        iter_type: Type,
        iter_name: Ident,
        array: Box<Expr>,
        body: Block,
    },
    Expr(Box<Expr>),
    Error,
}

pub type Type = ItemWithSpan<InnerType>;
#[derive(Debug, Clone, PartialEq)]
pub enum InnerType {
    Int,
    Bool,
    String,
    Array(Box<InnerType>),
    Class(String),
    Null,
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
    FunCall {
        function_name: Ident,
        args: Vec<Box<Expr>>,
    },
    BinaryOp(Box<Expr>, BinaryOp, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
    NewArray {
        elem_type: Type,
        elem_cnt: Box<Expr>,
    },
    ArrayElem {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    NewObject(Type),
    ObjField {
        obj: Box<Expr>,
        field: Ident,
    },
    ObjMethodCall {
        obj: Box<Expr>,
        method_name: Ident,
        args: Vec<Box<Expr>>,
    },
}

pub type UnaryOp = ItemWithSpan<InnerUnaryOp>;
#[derive(Debug)]
pub enum InnerUnaryOp {
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

impl fmt::Display for InnerType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::InnerType::*;
        match self {
            Int => write!(f, "int"),
            Bool => write!(f, "boolean"),
            String => write!(f, "string"),
            Array(subtype) => {
                subtype.fmt(f)?;
                write!(f, "[]")
            }
            Class(name) => write!(f, "{}", name),
            Null => write!(f, "null"),
            Void => write!(f, "void"),
        }
    }
}
