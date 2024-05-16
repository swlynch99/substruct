use substruct::substruct;

#[substruct(A, B, C)]
pub union A {
    #[substruct(A, B, C)]
    pub x: u64,
    #[substruct(A, B)]
    pub y: u32,
    #[substruct(A)]
    pub z: u16,
}

fn main() {
    let value = C { x: 77 };
    value.y = 1;
    value.z = 1;
}
