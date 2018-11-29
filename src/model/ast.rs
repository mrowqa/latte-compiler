type Span = (usize, usize);

pub struct ItemWithSpan<T> {
    inner: T,
    span: Span,
}

pub struct Program {
    functions: Vec<Function>,
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

pub type Stmt = ItemWithSpan<InnerStmt>;
pub enum InnerStmt {
    Empty,
    Block(Block),
    Decl{var_type: Type, var_items: Vec<(String, Option<Expr>)>},
    Assign(String, Expr),
    Ret(Option<Expr>),
    Cond{cond: Expr, true_branch: Box<Stmt>, false_branch: Option<Box<Stmt>>},
    While(Expr, Box<Stmt>),
    Expr(Expr),
}

pub type Type = ItemWithSpan<InnerType>;
pub enum InnerType {
    Int,
    Bool,
    String,
    Void,
}

pub type Expr = ItemWithSpan<InnerExpr>;
pub enum InnerExpr {
    LitVar(String),
    LitInt(i32),
    LitBool(bool),
    LitStr(String),
    FunCall{function_name: String, args: Vec<Expr>},
    BinaryOp(Box<Expr>, BinaryOp, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
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
