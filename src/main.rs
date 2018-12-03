extern crate latte_compiler;

use latte_compiler::parser::parse_or_string_error;
use latte_compiler::codemap::CodeMap;


fn main() {
    let res = parse_or_string_error(&CodeMap::new("input", r#"
int main () {
  Counter c;
  if
  c = new Counter; ff
  c.incr();
  c.incr();
  c.incr();
  int x = c.value();
  printInt(x);
  return 0;
  5.[haha]; // test :D
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
  Point2 p = new Point3;
  Point3 q = new Point3;
  Point4 r = new Point4;

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
  Shape elem;
  Node next;

  void setElem(Shape c) { elem = c; }

  void setNext(Node n) { next = n; }

  Shape getElem() { return elem; }

  Node getNext() { return next; }
}

class Stack {
  Node head;

  void push(Shape c) {
    Node newHead = new Node;
    newHead.setElem(c);
    newHead.setNext(head);
    head = newHead;
  }

  boolean isEmpty() {
    return head==null;
  }

  Shape top() {
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
  Stack stk = new Stack;
  Shape s = new Shape;
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
