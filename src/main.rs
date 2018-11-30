extern crate latte_compiler;


fn main() {
    let x = latte_compiler::parse("1 || 2 && 3");
    println!("{:?}", x);
}
