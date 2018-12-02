lalrpop_mod!(latte);
use self::latte::ProgramParser;
use model;
use model::ast::Span;
use codemap::CodeMap;

#[derive(Debug)]
pub struct ParseError {
    pub err: &'static str,
    pub span: Span,
}

pub fn parse(codemap: &CodeMap) -> Result<model::ast::Program, Vec<ParseError>> {
    let code = replace_comments(codemap.get_code());
    let code = match code {
        Ok(without_comments) => without_comments,
        Err(err) => return Err(vec![err]),
    };

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

pub fn parse_or_string_error(codemap: &CodeMap) -> Result<model::ast::Program, String> {
    match parse(codemap) {
        Ok(prog) => Ok(prog),
        Err(errors) => {
            let mut result = String::new();
            for ParseError { err, span } in errors {
                let msg = codemap.format_message(span, err);
                result.push_str(&msg);
            }
            Err(result)
        }
    }
}

fn replace_comments(code: &str) -> Result<String, ParseError> {
    let mut result = String::new();

    let mut last_ch = '\0';
    let mut erasing = false;
    let mut multiline = false;
    let mut inside_string = false;
    for ch in code.chars() {
        if !erasing {  // check if comment begins
            match (inside_string, last_ch, ch) {
                (false, _, '"') => {
                    inside_string = true;
                    result.push(ch);
                },
                (true, _, '"') if last_ch != '\\' => {
                    inside_string = false;
                    result.push(ch);
                },
                (true, _, _) => {
                    result.push(ch);
                },
                (false, _, '#') | (false, '/', '/') => {
                    erasing = true;
                    multiline = false;

                    if last_ch == '/' {
                        result.pop();
                        result.push(' ');
                    }
                    result.push(' ');
                },
                (false, '/', '*') => {
                    erasing = true;
                    multiline = true;
                    result.pop();
                    result.push_str("  ");
                },
                _ => {
                    result.push(ch);
                },
            }
        }
        else {  // check if comments ends
            match (multiline, last_ch, ch) {
                (false, _, '\n') => {
                    erasing = false;
                    result.push(ch);
                },
                (true, '*', '/') => {
                    erasing = false;
                    result.push(' ');
                },
                _ => {
                    result.push(if ch == '\n' {'\n'} else {' '});
                }
            }
        }

        last_ch = ch;
    }

    if erasing && multiline {
        Err(ParseError{
            err: "Multiline comment must be closed before EOF",
            span: (code.len() - 1, code.len()),
        })
    }
    else {
        Ok(result)
    }
}