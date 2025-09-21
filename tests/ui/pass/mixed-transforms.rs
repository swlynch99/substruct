use substruct::substruct;

#[substruct(ConvertedParams)]
#[derive(Clone, Debug)]
pub struct OriginalParams {
    #[substruct(ConvertedParams(unwrap))]
    pub name: Option<String>,

    #[substruct(ConvertedParams(try_into = u64))]
    pub id: u32,

    #[substruct(ConvertedParams)]
    pub active: bool,
}

fn main() {
    // Minimal compilation test - just verify mixed transforms generate valid code
    let original = OriginalParams {
        name: Some("test".to_string()),
        id: 42u32,
        active: true,
    };

    let _converted = ConvertedParams::try_from(original).unwrap();
}
