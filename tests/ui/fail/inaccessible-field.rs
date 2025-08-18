mod inner {
    use substruct::substruct;

    #[substruct(A)]
    #[derive(Default)]
    pub struct Test {
        #[substruct(pub(self) A)]
        pub field1: u32,
        pub field2: u32,
    }
}

fn main() {
    use self::inner::*;

    let value = A::default();
    let _field = value.field1;
}
