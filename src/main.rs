extern crate latte_compiler;

use latte_compiler::compile;

fn main() {
    // todo run on actual file
    let res = compile(
        "input",
        r#"
int main(int arg) {
    return 0;
}

class A {
    int a;

    int foo(int b) {
        return a+b+c;  // c not defined
    }
}

"#,
    );
    match res {
        Ok(prog) => println!("{:?}", prog),
        Err(msg) => println!("{}", msg),
    }
}
