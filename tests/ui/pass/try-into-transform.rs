use substruct::substruct;

#[substruct(ConvertedParams)]
#[derive(Clone, Debug)]
pub struct OriginalParams {
    #[substruct(ConvertedParams(try_into = u32))]
    pub id: u8,

    #[substruct(ConvertedParams)]
    pub limit: usize,
}

fn main() {
    // Minimal compilation test - just verify the generated code compiles
    let original = OriginalParams {
        id: 42u8,
        limit: 10,
    };

    let _converted = ConvertedParams::try_from(original).unwrap();
}
