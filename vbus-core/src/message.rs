use std::sync::Arc;
use std::time::Duration;
use crate::payload::Payload;

pub struct Message<T: Payload> {
    message_data: Arc<MessageData<T>>,
}

struct MessageData<T: Payload> {
    time_stamp: Duration,
    payload: T,
}

impl<T: Payload> Message<T> {
    pub fn new(time_stamp: Duration, payload: T) -> Self {
        Self {
            message_data: Arc::<MessageData<T>>::new(MessageData::new(time_stamp, payload)),
        }
    }

    pub fn get_type_stamp(&self) -> Duration {
        self.message_data.time_stamp.clone()
    }

    pub fn get_payload(&self) -> &T {
        &self.message_data.payload
    }
}

impl<T: Payload> Clone for Message<T> {
    fn clone(&self) -> Self {
        Self {
            message_data: self.message_data.clone(),
        }
    }
}

impl<T: Payload> MessageData<T> {
    fn new(time_stamp: Duration, payload: T) -> Self {
        Self {
            time_stamp,
            payload,
        }
    }
}
