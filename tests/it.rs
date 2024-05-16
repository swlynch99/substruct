use substruct::substruct;

#[test]
fn test_convert_tuple() {
    #[substruct(B)]
    struct A(pub i32, #[substruct(B)] pub i64);

    let b = B(32);
    let a = b.into_a(5);

    assert!(matches!(a, A(5, 32)))
}

#[test]
fn test_convert_normal() {
    #[substruct(B)]
    struct A {
        #[substruct(B)]
        pub field1: i32,
        pub field2: u32,
    }

    let b = B { field1: 1 };
    let a = b.into_a(7);

    assert!(matches!(
        a,
        A {
            field1: 1,
            field2: 7
        }
    ));
}
