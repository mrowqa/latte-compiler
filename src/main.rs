extern crate latte_compiler;

use latte_compiler::compile;


fn main() {
    let res = compile("input", r#"
int main() {
    return 0;
}

int error() {}
void
readInt(int input) {}

"#);
    match res {
        Ok(prog) => println!("{:?}", prog),
        Err(msg) => println!("{}", msg),
    }
}
