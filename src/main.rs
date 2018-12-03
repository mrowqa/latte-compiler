extern crate latte_compiler;

use latte_compiler::parser::parse_or_string_error;
use latte_compiler::codemap::CodeMap;


fn main() {
    let res = parse_or_string_error(&CodeMap::new("input", r#"
int main () {
  class Counter c;
  c = new Counter;
  c.incr();
  c.incr();
  c.incr();
  int x = c.value();
  printInt(x);
  return 0;
}

class Counter {
  int val;

  void incr () {val++; return;}
  int value () {return val;}
}

// ----------------------------

class Point2 {
  int x;
  int y;

  void move (int dx, int dy) {
     x = x + dx;
     y = y + dy;
  }

  int getX () { return x; }

  int getY () { return y; }
}

class Point3 extends Point2 {
  int z;

  void moveZ (int dz) {
    z = z + dz;
  }

  int getZ () { return z; }
}

class Point4 extends Point3 {
  int w;

  void moveW (int dw) {
    w = w + dw;
  }

  int getW () { return w; }
}

int main () {
  class Point2 p = new Point3;
  class Point3 q = new Point3;
  class Point4 r = new Point4;

  q.move(2,4);
  q.moveZ(7);
  p = q;

  p.move(3,5);

  r.move(1,3);
  r.moveZ(6);
  r.moveW(2);

  printInt(p.getX());
  printInt(p.getY());
  printInt(q.getZ());
  printInt(r.getW());
  return 0;
}

// --------------------------------------------

class Node {
  class Shape elem;
  class Node next;

  void setElem(class Shape c) { elem = c; }

  void setNext(class Node n) { next = n; }

  class Shape getElem() { return elem; }

  class Node getNext() { return next; }
}

class Stack {
  class Node head;

  void push(class Shape c) {
    class Node newHead = new Node;
    newHead.setElem(c);
    newHead.setNext(head);
    head = newHead;
  }

  boolean isEmpty() {
    return head==null;
  }

  class Shape top() {
    return head.getElem();
  }

  void pop() {
    head = head.getNext();
  }
}

class Shape {
  void tell () {
    printString("I'm a shape");
  }

  void tellAgain() {
     printString("I'm just a shape");
  }
}

class Rectangle extends Shape {
  void tellAgain() {
    printString("I'm really a rectangle");
  }
}

class Circle extends Shape {
  void tellAgain() {
    printString("I'm really a circle");
  }
}

class Square extends Rectangle {
  void tellAgain() {
    printString("I'm really a square");
  }
}

int main() {
  class Stack stk = new Stack;
  class Shape s = new Shape;
  stk.push(s);
  s = new Rectangle;
  stk.push(s);
  s = new Square;
  stk.push(s);
  s = new Circle;
  stk.push(s);
  while (!stk.isEmpty()) {
    s = stk.top();
    s.tell();
    s.tellAgain();
    stk.pop();
  }
  return 0;
}

"#));
    match res {
        Ok(prog) => println!("{:?}", prog),
        Err(msg) => println!("{}", msg),
    }
}
