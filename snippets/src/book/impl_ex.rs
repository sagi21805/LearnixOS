struct A(u32);

impl A {
    pub fn new(a: u32) -> A {
        A(a)
    }
}

struct B(u32);

impl B {
    pub fn new(b: u32) -> B {
        B(b)
    }
}
