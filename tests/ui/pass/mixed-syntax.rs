use substruct::substruct;

#[substruct(A(unwrap), B, C(unwrap))]
#[derive(Clone, Debug)]
pub struct MixedSyntax {
    #[substruct(A(unwrap), B, C(unwrap))]
    pub field1: Option<String>,

    #[substruct(A, B, C)]
    pub field2: i32,

    #[substruct(B)]
    pub field3: bool,

    pub field4: Option<f64>,
}

fn main() {
    // Test struct A with unwrap transformation
    let mixed = MixedSyntax {
        field1: Some("test".to_string()),
        field2: 42,
        field3: true,
        field4: Some(std::f64::consts::PI),
    };

    let a = A::try_from(mixed.clone()).unwrap();
    assert_eq!(a.field1, "test"); // String, not Option<String>
    assert_eq!(a.field2, 42);

    // Test struct B without unwrap
    let b = B::from(mixed.clone());
    assert_eq!(b.field1, Some("test".to_string())); // Still Option<String>
    assert_eq!(b.field2, 42);
    assert!(b.field3);

    // Test struct C with unwrap transformation
    let c = C::try_from(mixed).unwrap();
    assert_eq!(c.field1, "test"); // String, not Option<String>
    assert_eq!(c.field2, 42);

    // Test failed conversion when Option is None
    let mixed_none = MixedSyntax {
        field1: None,
        field2: 42,
        field3: true,
        field4: Some(std::f64::consts::PI),
    };

    // A and C should fail because they require unwrap
    assert!(A::try_from(mixed_none.clone()).is_err());
    assert!(C::try_from(mixed_none.clone()).is_err());

    // B should succeed because it doesn't unwrap
    let b_none = B::from(mixed_none);
    assert_eq!(b_none.field1, None);
}
