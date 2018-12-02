extern crate latte_compiler;
use latte_compiler::parser::parse_or_string_error;
use latte_compiler::codemap::CodeMap;


fn main() {
    let res = parse_or_string_error(&CodeMap::new("input", r#"
int main () {
    #1;
   /* if (true) {
        hehe */
    #6;
}
int main2() {/*"string //";* /}
"#));
    match res {
        Ok(prog) => println!("{:?}", prog),
        Err(msg) => println!("{}", msg),
    }
}
