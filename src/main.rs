extern crate latte_compiler;


fn main() {
    // parses incorrectly -- because using empty lexem ""?
    let x = latte_compiler::parser::parse(r#"
int main () {
    printString("Hel/*lo, worl*/d!\n");
    printString("Hello, world!\n");
}
int main2() {}
"#);
    println!("{:?}", x);
}
