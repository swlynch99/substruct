use substruct::substruct;

#[substruct(B, C, D)]
pub struct A {
    #[substruct_attr(blah, blah, blah)]
    #[substruct_attr(blah, bleh, blah)]
    pub x: u32
}

fn main() {}
