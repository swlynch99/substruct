use substruct::substruct;

#[substruct(RequiredTuple)]
#[derive(Clone, Debug)]
pub struct OptionalTuple(
    #[substruct(RequiredTuple)] pub String,
    #[substruct(RequiredTuple(unwrap))] pub Option<i32>,
    pub bool,
);

fn main() {
    let optional = OptionalTuple("test".to_string(), Some(42), true);
    let required = RequiredTuple::try_from(optional).unwrap();

    assert_eq!(required.0, "test");
    assert_eq!(required.1, 42);

    // Test failed conversion
    let optional_none = OptionalTuple("test".to_string(), None, true);
    assert!(RequiredTuple::try_from(optional_none).is_err());
}
