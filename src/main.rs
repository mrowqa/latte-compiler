extern crate latte_compiler;

use latte_compiler::parser::parse_or_string_error;
use latte_compiler::codemap::CodeMap;


fn main() {
    let res = parse_or_string_error(&CodeMap::new("input", r#"
int main () {
    if (1) while (3) if (2) ; else if (5) ; else ;
}

"#));
    match res {
        Ok(prog) => println!("{:?}", prog),
        Err(msg) => println!("{}", msg),
    }
}
