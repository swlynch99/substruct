#![deny(clippy::too_many_arguments)]
#![allow(dead_code)]

use substruct::substruct;

#[substruct(A, B)]
struct Large {
    #[substruct(B)]
    pub v0: u32,
    #[substruct(B)]
    pub v1: u32,
    #[substruct(B)]
    pub v2: u32,
    #[substruct(B)]
    pub v3: u32,
    #[substruct(B)]
    pub v4: u32,
    #[substruct(B)]
    pub v5: u32,
    #[substruct(B)]
    pub v6: u32,
    #[substruct(B)]
    pub v7: u32,
    #[substruct(B)]
    pub v8: u32,
    pub v9: u32,
}

fn main() {}
