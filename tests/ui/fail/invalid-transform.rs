use substruct::substruct;

#[substruct(B)]
struct A {
    #[substruct(B(invalid_transform))]
    field: Option<String>,
}

fn main() {}
