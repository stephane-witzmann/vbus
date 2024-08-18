use crate::payload::Payload;
use crate::message::Message;
use crate::queue::Queue;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Duration, Instant};

pub struct Channel<T: Payload> {
    t0: Instant,
    channel_rwlock: Arc<RwLock<ChannelData<T>>>,
}

struct ChannelData<T: Payload> {
    consumer_queues: Vec<Queue<T>>,
}

pub struct Producer<T: Payload> {
    channel: Channel<T>,
    t0: Instant,
}

pub struct Consumer<T: Payload> {
    channel: Channel<T>,
    queue: Queue<T>,
}

impl<T: Payload> Channel<T> {
    pub fn new(t0: Instant) -> Self {
        Self {
            channel_rwlock: Arc::new(RwLock::new(ChannelData::new())),
            t0,
        }
    }

    pub fn new_producer(&self) -> Producer<T> {
        Producer::new(self.clone(), self.t0)
    }

    pub fn new_consumer(&self) -> Consumer<T> {
        let mut data = self.write();

        let consumer = Consumer::new(self.clone());
        data.consumer_queues.push(consumer.queue.clone());

        consumer
    }

    pub fn now(&self) -> Duration {
        self.t0.elapsed()
    }

    fn broadcast(&self, message: Message<T>) {
        self.channel_rwlock.read().unwrap().broadcast(message);
    }

    fn read(&self) -> RwLockReadGuard<'_, ChannelData<T>> {
        self.channel_rwlock.read().unwrap()
    }

    fn write(&self) -> RwLockWriteGuard<'_, ChannelData<T>> {
        self.channel_rwlock.write().unwrap()
    }
}

impl<T: Payload> Clone for Channel<T> {
    fn clone(&self) -> Self {
        Self {
            channel_rwlock: self.channel_rwlock.clone(),
            t0: self.t0
        }
    }
}

impl<T: Payload> ChannelData<T> {
    fn new() -> Self {
        Self {
            consumer_queues: Vec::<Queue<T>>::new(),
        }
    }

    fn broadcast(&self, message: Message<T>) {
        for queue in &self.consumer_queues {
            queue.push(message.clone());
        }
    }
}

impl<T: Payload> Producer<T> {
    fn new(channel: Channel<T>, t0: Instant) -> Self {
        Self { channel, t0 }
    }

    pub fn push(&self, payload: T) {
        self.push_message(Message::new(self.channel.now(), payload));
    }

    fn push_message(&self, message: Message<T>) {
        self.channel.broadcast(message);
    }

    pub fn channel(&self) -> &Channel<T> {
        &self.channel
    }
}

impl<T: Payload> Consumer<T> {
    fn new(channel: Channel<T>) -> Self {
        Self {
            channel,
            queue: Queue::new(),
        }
    }

    pub fn interrupt_wait_pull(&self) {
        self.queue.wake_up();
    }

    pub fn wait_pull(&self) -> Vec<Message<T>> {
        self.queue.wait_pull()
    }

    pub fn pull(&self) -> Vec<Message<T>> {
        self.queue.pull()
    }

    pub fn channel(&self) -> &Channel<T> {
        &self.channel
    }
}

impl<T: Payload> Drop for Consumer<T> {
    fn drop(&mut self) {
        let mut lock = self.channel.write();
        lock.consumer_queues.retain(|queue| !(queue == &self.queue));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use crate::test_payload::*;

    #[test]
    fn test_instantiate_producers() {
        let t0 = Instant::now();
        let channel = Channel::<TestPayload>::new(t0);

        let n: usize = 10;
        let mut v = Vec::<Producer<TestPayload>>::new();
        for _ in 0..n {
            v.push(channel.new_producer());
        }
    }

    #[test]
    fn test_instantiate_consumers() {
        let t0 = Instant::now();
        let channel = Channel::<TestPayload>::new(t0);

        let n: usize = 10;
        let mut v = Vec::<Consumer<TestPayload>>::new();
        for _ in 0..n {
            v.push(channel.new_consumer());
        }
        assert_eq!(v.len(), n);
        assert_eq!(v.first().unwrap().channel.read().consumer_queues.len(), n);
    }

    #[test]
    fn test_drop_consumer() {
        let t0 = Instant::now();
        let channel = Channel::<TestPayload>::new(t0);

        let n: usize = 10;
        let mut v = Vec::<Consumer<TestPayload>>::new();
        for i in 0..n {
            assert_eq!(channel.channel_rwlock.read().unwrap().consumer_queues.len(), i);
            v.push(channel.new_consumer());
            channel.new_producer().push(TestPayload::new(i)); // so that we can identify consumers
        }

        assert_eq!(channel.channel_rwlock.read().unwrap().consumer_queues.len(), n);

        for i in 0..n {
            assert_eq!(v.pop().unwrap().queue.pull().len(), i + 1); // make sure that we're popping the right consumer
            assert_eq!(channel.channel_rwlock.read().unwrap().consumer_queues.len(), n - i - 1);
        }
    }

    #[test]
    fn test_consumer_api() {
        let t0: Instant = Instant::now();
        let channel = Channel::<TestPayload>::new(t0);
        let consumer = channel.new_consumer();

        assert_eq!(consumer.queue.len(), 0);
        assert_eq!(consumer.pull().len(), 0);

        consumer.queue.push(Message::new(channel.now(), TestPayload::new(0)));
        assert_eq!(consumer.queue.len(), 1);
        consumer.queue.push(Message::new(channel.now(), TestPayload::new(0)));
        assert_eq!(consumer.queue.len(), 2);
        assert_eq!(consumer.pull().len(), 2);
        assert_eq!(consumer.queue.len(), 0);

        consumer.queue.push(Message::new(channel.now(), TestPayload::new(0)));
        assert_eq!(consumer.queue.len(), 1);
        assert_eq!(consumer.wait_pull().len(), 1);
        assert_eq!(consumer.queue.len(), 0);

        let wait_time = Duration::from_millis(200);
        let queue_for_thread = consumer.queue.clone();
        let msg = Message::new(channel.now(), TestPayload::new(0));
        let now = Instant::now();
        let _ = std::thread::spawn(move || {
            sleep(wait_time);
            queue_for_thread.push(msg)
        });
        assert_eq!(consumer.wait_pull().len(), 1);
        let elapsed = now.elapsed();
        assert!(elapsed >= wait_time);
    }
}
