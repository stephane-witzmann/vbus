use crate::Payload;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Instant;

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
    pub(crate) fn new(time_stamp: Instant, payload: T) -> Self {
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

impl<T: Payload> Deref for Message<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get_payload()
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
