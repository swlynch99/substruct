mod inner {
    use substruct::substruct;

    #[substruct(pub(self) A)]
    #[derive(Default)]
    pub struct Test {
        #[substruct(A)]
        pub field: u32,
    }
}

fn main() {
    let _ = crate::inner::A::default();
}
