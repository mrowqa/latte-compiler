lalrpop_mod!(latte);
use self::latte::ProgramParser;
use model;

// todo support returning errors
pub fn parse(code: &str) -> model::ast::Program {
    //let code = _replace_comments(code);
    let parser = ProgramParser::new();
    parser.parse(&code).unwrap()
}

// todo shouldn't look inside strings...
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
