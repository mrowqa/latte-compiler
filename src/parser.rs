lalrpop_mod!(latte);
use self::latte::ProgramParser;
use model;
use model::ast::Span;
use codemap::format_message;

#[derive(Debug)]
pub struct ParseError {
    pub err: &'static str,
    pub span: Span,
}

// todo support returning errors
pub fn parse(code: &str) -> Result<model::ast::Program, Vec<ParseError>> {
    //let code = _replace_comments(code);
    let mut errors = Vec::new();
    let result = ProgramParser::new().parse(&mut errors, &code);
    match result {
        Ok(program) => {
            if errors.is_empty() { // probably must be empty
                Ok(program)
            }
            else {
                Err(errors)
            }
        },
        Err(_) => {
            if errors.is_empty() { // probably mustn't be empty
                errors.push(ParseError {
                    err: "Fatal syntax error: can not recognize anything",
                    span: (0, code.len() - 1),
                });
            }
            Err(errors)
        }
    }
}

pub fn parse_or_string_error(filename: &str, code: &str) -> Result<model::ast::Program, String> {
    match parse(code) {
        Ok(prog) => Ok(prog),
        Err(errors) => {
            let mut result = String::new();
            for ParseError { err, span } in errors {
                let msg = format_message(filename, span, code, err);
                result.push_str(&msg);
            }
            Err(result)
        }
    }
}

// todo comment remover shouldn't look inside strings...
fn _replace_comments(code: &str) -> String {
    let mut result = String::new();

    let mut last_ch = '\0';
    let mut erasing = false;
    let mut multiline = false;
    for ch in code.chars() {
        if !erasing { // check if comment begins
            if ch == '#' || last_ch == '/' && ch == '/' {
                erasing = true;
                multiline = false;

                if last_ch == '/' {
                    result.pop();
                    result.push(' ');
                }
                result.push(' ');
            }
            else if last_ch == '/' && ch == '*' {
                erasing = true;
                multiline = true;
                result.pop();
                result.push_str("  ");
            }
            else {
                result.push(ch);
            }
        }
        else { // check if comments ends
            if !multiline && ch == '\n' {
                erasing = false;
                result.push(ch);
            }
            else if multiline && last_ch == '*' && ch == '/' {
                erasing = false;
                result.push(' ');
            }
            else {
                result.push(if ch == '\n' {'\n'} else {' '});
            }
        }

        last_ch = ch;
    }

    result
}
