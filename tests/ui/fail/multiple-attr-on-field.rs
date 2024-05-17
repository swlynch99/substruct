use substruct::substruct;

#[substruct(B, C)]
struct A {
    #[substruct(B)]
    #[substruct(C)]
    field: u32,

}

fn main() {}
