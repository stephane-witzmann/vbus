pub trait Payload where Self: Sized {
    fn format_name() -> &'static str;

    fn to_binary(&self) -> Vec<u8>;

    fn from_binary(input: &Vec<u8>) -> Option<Self>;
}
