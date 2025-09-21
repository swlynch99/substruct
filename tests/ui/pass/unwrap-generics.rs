use substruct::substruct;

#[substruct(RequiredGeneric)]
#[derive(Clone, Debug)]
pub struct OptionalGeneric<T> {
    #[substruct(RequiredGeneric(unwrap))]
    pub value: Option<T>,

    #[substruct(RequiredGeneric)]
    pub id: usize,

    pub metadata: String,
}

fn main() {
    // Test with String type
    let optional = OptionalGeneric {
        value: Some("hello".to_string()),
        id: 42,
        metadata: "test".to_string(),
    };

    let required = RequiredGeneric::try_from(optional).unwrap();
    assert_eq!(required.value, "hello");
    assert_eq!(required.id, 42);

    // Test conversion back
    let back = required.into_optional_generic("new_metadata".to_string());
    assert_eq!(back.value, Some("hello".to_string()));
    assert_eq!(back.metadata, "new_metadata");

    // Test with failed conversion
    let optional_none: OptionalGeneric<String> = OptionalGeneric {
        value: None,
        id: 42,
        metadata: "test".to_string(),
    };

    assert!(RequiredGeneric::try_from(optional_none).is_err());
}
