use crate::Error;
use crate::Message;
use crate::Payload;
use crate::Player;
use crate::Recorder;
use crate::ThreadedConsumer;
use crate::private::Consumer;
use crate::private::queue::Queue;
use std::path::Path;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::Instant;

pub struct Channel<T: Payload> {
    data: Arc<ChannelData<T>>,
}

impl<T: Payload> PartialEq for Channel<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
    }
}

impl<T: Payload> Eq for Channel<T> {}

impl<T: Payload> Clone for Channel<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<T: Payload> Default for Channel<T> {
    fn default() -> Self {
        Self {
            data: Arc::new(ChannelData {
                queues: RwLock::new(Vec::<Queue<T>>::new()),
            }),
        }
    }
}

impl<T: Payload> Channel<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&self, payload: T) {
        self.push_message(Message::new(Instant::now(), payload));
    }

    pub(crate) fn push_message(&self, message: Message<T>) {
        self.broadcast(message);
    }

    pub(crate) fn new_consumer(&self) -> Consumer<T> {
        Consumer::new(self)
    }

    pub fn new_threaded_consumer(
        &self,
        process: impl FnMut(Vec<Message<T>>) + Send + 'static,
    ) -> ThreadedConsumer<T> {
        let consumer = self.new_consumer();
        ThreadedConsumer::new(consumer, process)
    }

    pub fn new_recorder(&self, path: &Path) -> Result<Recorder<T>, Error> {
        Recorder::<T>::new(self, path)
    }

    pub fn new_player(&self, path: &Path) -> Result<Player<T>, Error> {
        Player::<T>::new(self, path)
    }

    fn broadcast(&self, message: Message<T>) {
        for queue in self.queues_read().iter() {
            queue.push(message.clone());
        }
    }

    fn queues_read(&self) -> RwLockReadGuard<'_, Vec<Queue<T>>> {
        self.data.queues.read().unwrap()
    }

    fn queues_write(&self) -> RwLockWriteGuard<'_, Vec<Queue<T>>> {
        self.data.queues.write().unwrap()
    }

    pub(crate) fn add_queue(&self, queue: &Queue<T>) {
        self.queues_write().push(queue.clone());
    }

    pub(crate) fn remove_queue(&self, queue: &Queue<T>) {
        self.queues_write().retain(|q| q != queue);
    }

    #[cfg(test)]
    pub(crate) fn queues_len(&self) -> usize {
        self.queues_read().len()
    }
}

struct ChannelData<T: Payload> {
    queues: RwLock<Vec<Queue<T>>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::test_tools::{EmptyPayload, TestPayload};

    #[test]
    fn test_channel_eq() {
        let a = Channel::<EmptyPayload>::new();
        assert!(a == a);

        let a_clone = a.clone();
        assert!(a == a_clone);

        let b = Channel::<EmptyPayload>::new();
        assert!(a != b);
    }

    #[test]
    fn test_channel_global() {
        let channel: Channel<TestPayload> = Channel::<TestPayload>::new();
        let _: Channel<TestPayload> = channel.clone();

        channel.push(TestPayload::default());

        let consumer: Consumer<TestPayload> = channel.new_consumer();
        let _: &Channel<TestPayload> = consumer.channel();

        let vec_content = [123456789, 42, 0];

        for content in vec_content {
            let start_time = Instant::now();
            channel.push(TestPayload::new(content));

            let messages = consumer.pull();
            let stop_time = Instant::now();

            let message = messages.first().unwrap();
            message.get_payload().check(content);

            let push_time = message.get_type_stamp();
            assert!(push_time > start_time);
            assert!(stop_time > push_time);
        }
    }
}
