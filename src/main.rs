extern crate latte_compiler;

use latte_compiler::parser::parse_or_string_error;
use latte_compiler::codemap::CodeMap;


fn main() {
    let res = parse_or_string_error(&CodeMap::new("input", r#"
class list {
  int elem;
  class list next;
}

int main() {
  printInt(length(fromTo(1,50)));
  printInt(length2(fromTo(1,100)));
}

int head (class list xs) {
  return xs . elem;
}

class list cons (int x, class list xs) {
  class list n;
  n = new list;
  n.elem = x;
  n.next = xs;
  return n;
}

int length (class list xs) {
  if (xs==null)
    return 0;
  else
    return 1 + length (xs.next);
}

class list fromTo (int m, int n) {
  if (m>n)
    return null;
  else
    return cons (m,fromTo (m+1,n));
}

int length2 (class list xs) {
  int res = 0;
  while (xs != null) {
    res++;
    xs = xs.next;
  }
  return res;
}

"#));
    match res {
        Ok(prog) => println!("{:?}", prog),
        Err(msg) => println!("{}", msg),
    }
}
