use substruct::substruct;

#[test]
fn test_convert_tuple() {
    #[substruct(B)]
    struct A(pub i32, #[substruct(B)] pub i64);

    let b = B(32);
    let a = b.into_a(5);

    assert!(matches!(a, A(5, 32)));
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

#[test]
fn test_try_from_unwrap() {
    #[substruct(RequiredData)]
    #[derive(Clone, Debug, PartialEq)]
    struct OptionalData {
        #[substruct(RequiredData(unwrap))]
        pub name: Option<String>,

        #[substruct(RequiredData)]
        pub id: u32,

        pub metadata: Option<String>,
    }

    // Test successful conversion
    let optional = OptionalData {
        name: Some("test".to_string()),
        id: 42,
        metadata: Some("extra".to_string()),
    };

    let required = RequiredData::try_from(optional).unwrap();
    assert_eq!(required.name, "test");
    assert_eq!(required.id, 42);

    // Test round-trip conversion
    let back = required.into_optional_data(Some("new_extra".to_string()));
    assert_eq!(back.name, Some("test".to_string()));
    assert_eq!(back.id, 42);
    assert_eq!(back.metadata, Some("new_extra".to_string()));

    // Test failed conversion
    let optional_none = OptionalData {
        name: None,
        id: 42,
        metadata: Some("extra".to_string()),
    };

    let result = RequiredData::try_from(optional_none);
    assert!(result.is_err());
}

#[test]
fn test_try_into_transform() {
    use substruct::substruct;

    #[substruct(ConvertedParams)]
    #[derive(Clone, Debug)]
    pub struct OriginalParams {
        #[substruct(ConvertedParams(try_into = u64))]
        pub id: u32,

        #[substruct(ConvertedParams)]
        pub limit: usize,

        pub description: Option<String>,
    }

    // Test successful conversion
    let original = OriginalParams {
        id: 42u32,
        limit: 10,
        description: Some("test".to_string()),
    };

    let converted = ConvertedParams::try_from(original).unwrap();
    assert_eq!(converted.id, 42u64);
    assert_eq!(converted.limit, 10);

    // Test round-trip conversion
    let back_to_original = converted.into_original_params(Some("test".to_string()));
    assert_eq!(back_to_original.id, 42u32);
    assert_eq!(back_to_original.limit, 10);
    assert_eq!(back_to_original.description, Some("test".to_string()));
}

#[test]
fn test_mixed_transforms() {
    use substruct::substruct;

    #[substruct(MixedParams)]
    #[derive(Clone, Debug)]
    pub struct OriginalParams {
        #[substruct(MixedParams(unwrap))]
        pub name: Option<String>,

        #[substruct(MixedParams(try_into = u64))]
        pub id: u32,

        #[substruct(MixedParams)]
        pub active: bool,

        pub metadata: Option<String>,
    }

    // Test successful conversion with both transforms
    let original = OriginalParams {
        name: Some("test".to_string()),
        id: 123u32,
        active: true,
        metadata: Some("meta".to_string()),
    };

    let mixed = MixedParams::try_from(original).unwrap();
    assert_eq!(mixed.name, "test");
    assert_eq!(mixed.id, 123u64);
    assert!(mixed.active);

    // Test failed conversion due to None in unwrap
    let original_with_none = OriginalParams {
        name: None,
        id: 123u32,
        active: true,
        metadata: Some("meta".to_string()),
    };

    let result = MixedParams::try_from(original_with_none);
    assert!(result.is_err());

    // Test round-trip conversion
    let back_to_original = mixed.into_original_params(Some("meta".to_string()));
    assert_eq!(back_to_original.name, Some("test".to_string()));
    assert_eq!(back_to_original.id, 123u32);
    assert!(back_to_original.active);
    assert_eq!(back_to_original.metadata, Some("meta".to_string()));
}
