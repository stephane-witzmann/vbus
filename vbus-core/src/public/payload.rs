pub trait Payload
where
    Self: Sized + Send + Sync + bincode::Encode + bincode::Decode<()> + 'static,
{
    fn format_name() -> &'static str {
        std::any::type_name::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::test_tools::{EmptyPayload, TestPayload};

    #[test]
    fn test_payload_format_name() {
        assert_eq!(
            TestPayload::format_name(),
            std::any::type_name::<TestPayload>()
        );
        assert_eq!(
            EmptyPayload::format_name(),
            std::any::type_name::<EmptyPayload>()
        );
    }
}
