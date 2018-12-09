extern crate latte_compiler;

use latte_compiler::compile;

fn main() {
    let res = compile(
        "input",
        r#"
int main() {
    return 0;
}

void test(int a, boolean a) { }

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
