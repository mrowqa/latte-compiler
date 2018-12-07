extern crate latte_compiler;

use latte_compiler::compile;


fn main() {
    let res = compile("input", r#"
int main() {
    return 0;
}

class A { }
class B extends A { }
class C extends D { }
class E extends E { }
class X extends Y { }
class Y extends X { }

"#);
    match res {
        Ok(prog) => println!("{:?}", prog),
        Err(msg) => println!("{}", msg),
    }
}
