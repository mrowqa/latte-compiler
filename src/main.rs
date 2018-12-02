extern crate latte_compiler;


fn main() {
    // parses incorrectly -- because using empty lexem ""?
    let x = latte_compiler::parse(r#"
int main () {
    printString("Hello, world!\n");
    printString("Hello, world!\n");
}
int main2() {}
"#);
    println!("{:?}", x);
}
