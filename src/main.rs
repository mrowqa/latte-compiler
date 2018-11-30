extern crate latte_compiler;


fn main() {
    // parses incorrectly -- because using empty lexem ""?
    let x = latte_compiler::parse(r#"
int main () {
  printString("hello world");
}
int main2() {}
"#);
    println!("{:?}", x);
}
