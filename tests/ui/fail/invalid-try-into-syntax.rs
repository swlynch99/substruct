use substruct::substruct;

#[substruct(ConvertedParams)]
#[derive(Clone, Debug)]
pub struct OriginalParams {
    #[substruct(ConvertedParams(try_into))]
    pub id: u8,
}

fn main() {}
