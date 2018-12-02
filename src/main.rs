extern crate latte_compiler;

use latte_compiler::parser::parse_or_string_error;
use latte_compiler::codemap::CodeMap;


fn main() {
    let res = parse_or_string_error(&CodeMap::new("input", r#"
int main () {
  int[] a;
  string[] b;

  a = new int[20];
  int[] c = new int[30];

  for (int x : a)
    printInt(x);
}

int[] sum (int[] a, int[] b) {
  int[] res = new int [a.length];
  int i = 0;

  while (i < a.length) {
    res[i] = a[i] + b[i];
    i++;
  }
  return res;
}

"#));
    match res {
        Ok(prog) => println!("{:?}", prog),
        Err(msg) => println!("{}", msg),
    }
}
