mod inner {
    use substruct::substruct;

    #[substruct(A, B)]
    #[derive(Default)]
    pub struct Test {
        #[substruct(pub any(A, B))]
        field: u32,
    }
}

fn main() {
    use crate::inner::*;

    let a = A::default();
    let b = B::default();

    let _ = a.field;
    let _ = b.field;
}
