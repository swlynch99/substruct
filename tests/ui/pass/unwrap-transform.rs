use substruct::substruct;

#[substruct(RequiredParams)]
#[derive(Clone, Debug)]
pub struct OptionalParams {
    #[substruct(RequiredParams(unwrap))]
    pub name: Option<String>,

    #[substruct(RequiredParams)]
    pub limit: usize,

    pub description: Option<String>,
}

fn main() {
    // Test successful conversion
    let optional = OptionalParams {
        name: Some("test".to_string()),
        limit: 10,
        description: Some("desc".to_string()),
    };

    let required = RequiredParams::try_from(optional.clone()).unwrap();
    assert_eq!(required.name, "test");
    assert_eq!(required.limit, 10);

    // Test failed conversion
    let optional_none = OptionalParams {
        name: None,
        limit: 10,
        description: Some("desc".to_string()),
    };

    let result = RequiredParams::try_from(optional_none);
    assert!(result.is_err());

    // Test round-trip conversion
    let back_to_optional = required.into_optional_params(Some("desc".to_string()));
    assert_eq!(back_to_optional.name, Some("test".to_string()));
    assert_eq!(back_to_optional.limit, 10);
    assert_eq!(back_to_optional.description, Some("desc".to_string()));
}
