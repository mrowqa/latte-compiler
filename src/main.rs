extern crate latte_compiler;

use latte_compiler::compile;


fn main() {
    let res = compile("input", r#"
int main() {
    return 0;
}

void main() {}
void main(void a) {}

"#);
    match res {
        Ok(prog) => println!("{:?}", prog),
        Err(msg) => println!("{}", msg),
    }
}
