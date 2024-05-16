use substruct::substruct;

#[substruct(
    #[derive(Debug)]
    B
)]
pub struct A {
    pub x: u32,
}

fn main() {}
