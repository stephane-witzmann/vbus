use std::sync::Arc;
use std::time::Instant;

pub trait Payload
where
    Self: Sized + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static,
{
    fn format_name() -> &'static str {
        std::any::type_name::<Self>()
    }
}

pub struct Message<T: Payload> {
    message_data: Arc<MessageData<T>>,
}

impl<T: Payload> Clone for Message<T> {
    fn clone(&self) -> Self {
        Self {
            message_data: self.message_data.clone(),
        }
    }
}

impl<T: Payload> Message<T> {
    pub fn new(time_stamp: Instant, payload: T) -> Self {
        Self {
            message_data: Arc::<MessageData<T>>::new(MessageData::new(time_stamp, payload)),
        }
    }

    pub fn get_type_stamp(&self) -> Instant {
        self.message_data.time_stamp
    }

    pub fn get_payload(&self) -> &T {
        &self.message_data.payload
    }
}

struct MessageData<T: Payload> {
    time_stamp: Instant,
    payload: T,
}

impl<T: Payload> MessageData<T> {
    fn new(time_stamp: Instant, payload: T) -> Self {
        Self {
            time_stamp,
            payload,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payload_format_name() {
        assert_eq!(
            crate::test_payload::TestPayload::format_name(),
            std::any::type_name::<crate::test_payload::TestPayload>()
        );
        assert_eq!(
            crate::test_payload::EmptyPayload::format_name(),
            std::any::type_name::<crate::test_payload::EmptyPayload>()
        );
    }
}
