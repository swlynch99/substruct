use substruct::substruct;

#[substruct(AllRequired)]
#[derive(Clone, Debug)]
pub struct MultiOptional {
    #[substruct(AllRequired(unwrap))]
    pub first: Option<String>,

    #[substruct(AllRequired(unwrap))]
    pub second: Option<i32>,

    pub optional_field: Option<bool>,
}

fn main() {
    // Test successful conversion
    let multi = MultiOptional {
        first: Some("hello".to_string()),
        second: Some(42),
        optional_field: Some(true),
    };

    let required = AllRequired::try_from(multi).unwrap();
    assert_eq!(required.first, "hello");
    assert_eq!(required.second, 42);

    // Test failed conversion - first field None
    let multi_fail1 = MultiOptional {
        first: None,
        second: Some(42),
        optional_field: Some(true),
    };

    assert!(AllRequired::try_from(multi_fail1).is_err());

    // Test failed conversion - second field None
    let multi_fail2 = MultiOptional {
        first: Some("hello".to_string()),
        second: None,
        optional_field: Some(true),
    };

    assert!(AllRequired::try_from(multi_fail2).is_err());
}
