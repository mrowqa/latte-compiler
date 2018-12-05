extern crate latte_compiler;

use latte_compiler::parser::parse_or_string_error;
use latte_compiler::codemap::CodeMap;


fn main() {
    let res = parse_or_string_error(&CodeMap::new("input", r#"
int main() {
  true && false;
  true || false;
  "xD" + "haha";
  1 + 2;
  1 - 2;
  1 * 2;
  1 / 2;
  syntax error )
  1 / (3 *
     4 % 2) + 42;
  1 % 2;
  1 < 2;
  1 <= 2;
  1 > 2;
  1 >= 2;
  1 == 2;
  1 != 2;
  true == false;
  true != false;
  "xD" == "haha";
  "xD" != "haha";
  -42;
  - -42;
  !true;
  !!true;
  !!!true;
}

"#));
    match res {
        Ok(prog) => println!("{:?}", prog),
        Err(msg) => println!("{}", msg),
    }
}
