use substruct::substruct;

#[substruct(any(A, B, C))]
pub struct A {
    pub x: u32,
}

fn main() {}
