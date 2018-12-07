extern crate latte_compiler;

use latte_compiler::compile;


fn main() {
    let res = compile("input", r#"
int main() {
    return 0;
}

void main(void a) {}

NonExistingClass test(NonExistingClass a) {}

class A {
    NonExistingClass f; // shouldn't be detected since it's redefined later

    void a() {}
}

class A {
    NonExistingClass b; 
}

bool properFoo(int x) { x; }

"#);
    match res {
        Ok(prog) => println!("{:?}", prog),
        Err(msg) => println!("{}", msg),
    }
}
