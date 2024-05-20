use substruct::substruct;

#[substruct(
    #[derive(Debug)]
    B
)]
struct A {
    #[substruct(B)]
    field: u32,
}

fn main() {}
