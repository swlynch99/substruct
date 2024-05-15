use substruct::substruct;

#[substruct(B)]
struct A(pub i32, #[substruct(B)] pub i64);

#[test]
fn test_convert() {
    let b = B(32);
    let a = b.into_a(5);

    assert!(matches!(a, A(5, 32)))
}
