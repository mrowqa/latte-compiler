extern crate latte_compiler;


fn main() {
    let x = latte_compiler::parser::parse_or_string_error("input", r#"
int main () {
    #1;
   /* if (true) {
        hehe */
    #6;
}
int main2() {/*"string //";*/}
"#);
    match x {
        Ok(prog) => println!("{:?}", prog),
        Err(msg) => println!("{}", msg),
    }
}
