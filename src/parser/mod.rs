lalrpop_mod!(#[allow(clippy::all)] pub latte, "/parser/latte.rs");
use self::latte::ProgramParser;
use codemap::CodeMap;
use frontend_error::{FrontendError, FrontendResult};
use model::ast::{
    new_spanned_boxed, BinaryOp, Block, Expr, InnerExpr, InnerStmt, InnerUnaryOp, Program, Stmt,
};

const KEYWORDS: &[&str] = &[
    "if", "else", "return", "while", "for", "new", "class", "extends", "true", "false", "null",
    "int", "string", "boolean", "void",
];

pub fn parse(codemap: &CodeMap) -> FrontendResult<Program> {
    let code = replace_comments(codemap.get_code())?;

    let mut errors = Vec::new();
    let result = ProgramParser::new().parse(&mut errors, &code);
    match result {
        Ok(program) => {
            if errors.is_empty() {
                // probably must be empty
                Ok(program)
            } else {
                Err(errors)
            }
        }
        Err(_) => {
            if errors.is_empty() {
                // probably mustn't be empty
                errors.push(FrontendError {
                    err: "Fatal syntax error: can not recognize anything".to_string(),
                    span: (0, code.len() - 1),
                });
            }
            Err(errors)
        }
    }
}

fn replace_comments(code: &str) -> FrontendResult<String> {
    let mut result = String::new();

    let mut last_ch = '\0';
    let mut erasing = false;
    let mut multiline = false;
    let mut inside_string = false;
    for ch in code.chars() {
        if !erasing {
            // check if comment begins
            match (inside_string, last_ch, ch) {
                (false, _, '"') => {
                    inside_string = true;
                    result.push(ch);
                }
                (true, _, '"') if last_ch != '\\' => {
                    inside_string = false;
                    result.push(ch);
                }
                (true, _, _) => {
                    result.push(ch);
                }
                (false, _, '#') | (false, '/', '/') => {
                    erasing = true;
                    multiline = false;

                    if last_ch == '/' {
                        result.pop();
                        result.push(' ');
                    }
                    result.push(' ');
                }
                (false, '/', '*') => {
                    erasing = true;
                    multiline = true;
                    result.pop();
                    result.push_str("  ");
                }
                _ => {
                    result.push(ch);
                }
            }
        } else {
            // check if comments ends
            match (multiline, last_ch, ch) {
                (false, _, '\n') => {
                    erasing = false;
                    result.push(ch);
                }
                (true, '*', '/') => {
                    erasing = false;
                    result.push(' ');
                }
                _ => {
                    result.push(if ch == '\n' { '\n' } else { ' ' });
                }
            }
        }

        last_ch = ch;
    }

    if erasing && multiline {
        Err(vec![FrontendError {
            err: "Multiline comment must be closed before EOF".to_string(),
            span: (code.len() - 1, code.len()),
        }])
    } else {
        Ok(result)
    }
}

// ---------------------------- ----------------------
// --------------- parser utils ----------------------
// ---------------------------------------------------
fn optimize_const_expr_shallow(expr: InnerExpr) -> Result<InnerExpr, &'static str> {
    use self::BinaryOp::*;
    use self::InnerExpr::*;
    use self::InnerUnaryOp::*;
    let e = match expr {
        BinaryOp(ref lhs, ref op, ref rhs) => match (&lhs.inner, op, &rhs.inner) {
            (LitBool(l), And, LitBool(r)) => LitBool(*l && *r),
            (LitBool(l), Or, LitBool(r)) => LitBool(*l || *r),
            (LitStr(l), Add, LitStr(r)) => LitStr(l.to_string() + r),
            (LitInt(l), Add, LitInt(r)) => LitInt(l + r),
            (LitInt(l), Sub, LitInt(r)) => LitInt(l - r),
            (LitInt(l), Mul, LitInt(r)) => LitInt(l * r),
            (LitInt(l), Div, LitInt(r)) => {
                if *r == 0 {
                    return Err("Assertion Error: Division by zero in constant expression");
                }
                LitInt(l / r)
            }
            (LitInt(l), Mod, LitInt(r)) => LitInt(l % r),
            (LitInt(l), LT, LitInt(r)) => LitBool(l < r),
            (LitInt(l), LE, LitInt(r)) => LitBool(l <= r),
            (LitInt(l), GT, LitInt(r)) => LitBool(l > r),
            (LitInt(l), GE, LitInt(r)) => LitBool(l >= r),
            (LitInt(l), EQ, LitInt(r)) => LitBool(l == r),
            (LitInt(l), NE, LitInt(r)) => LitBool(l != r),
            (LitBool(l), EQ, LitBool(r)) => LitBool(l == r),
            (LitBool(l), NE, LitBool(r)) => LitBool(l != r),
            (LitStr(l), EQ, LitStr(r)) => LitBool(l == r),
            (LitStr(l), NE, LitStr(r)) => LitBool(l != r),
            _ => LitNull,
        },
        UnaryOp(ref op, ref subexpr) => match (&op.inner, &subexpr.inner) {
            (IntNeg, LitInt(l)) => LitInt(-l),
            (BoolNeg, LitBool(l)) => LitBool(!l),
            _ => LitNull,
        },
        _ => LitNull,
    };
    Ok(if let LitNull = e { expr } else { e })
}

fn return_or_fail(
    l: usize,
    result: Result<InnerExpr, &'static str>,
    r: usize,
    errors: &mut Vec<FrontendError>,
) -> Box<Expr> {
    match result {
        Ok(e) => new_spanned_boxed(l, e, r),
        Err(err) => {
            errors.push(FrontendError {
                err: err.to_string(),
                span: (l, r),
            });
            new_spanned_boxed(l, InnerExpr::LitNull, r)
        }
    }
}

fn stmt_to_block(stmt: Box<Stmt>) -> Block {
    if let InnerStmt::Block(bl) = stmt.inner {
        bl
    } else {
        let span = stmt.span;
        Block {
            stmts: vec![stmt],
            span,
        }
    }
}
