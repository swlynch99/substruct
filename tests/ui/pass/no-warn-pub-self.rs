#![allow(dead_code)]
#![deny(clippy::needless_pub_self)]

use substruct::substruct;

#[substruct(pub(self) A)]
pub struct Test {
    #[substruct(A)]
    pub field: u32,
}

fn main() {}
