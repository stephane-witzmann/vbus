use crate::message::{Message, Payload};
use crate::queue::Queue;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::Instant;

pub struct Channel<T: Payload> {
    data: Arc<ChannelData<T>>,
}

impl<T: Payload> Clone for Channel<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<T: Payload> Channel<T> {
    pub fn new() -> Self {
        Self {
            data: Arc::new(ChannelData {
                queues: RwLock::new(Vec::<Queue<T>>::new()),
            }),
        }
    }

    pub fn new_producer(&self) -> Producer<T> {
        Producer::new(self.clone())
    }

    pub fn new_consumer(&self) -> Consumer<T> {
        let consumer = Consumer::new(self.clone());
        self.queues_write().push(consumer.queue.clone());

        consumer
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
}

impl<T: Payload> Default for Channel<T> {
    fn default() -> Self {
        Self::new()
    }
}

struct ChannelData<T: Payload> {
    queues: RwLock<Vec<Queue<T>>>,
}

pub struct Producer<T: Payload> {
    channel: Channel<T>,
}

impl<T: Payload> Producer<T> {
    fn new(channel: Channel<T>) -> Self {
        Self { channel }
    }

    pub fn push(&self, payload: T) {
        self.push_message(Message::new(Instant::now(), payload));
    }

    fn push_message(&self, message: Message<T>) {
        self.channel.broadcast(message);
    }

    pub fn channel(&self) -> &Channel<T> {
        &self.channel
    }
}

pub struct Consumer<T: Payload> {
    channel: Channel<T>,
    queue: Queue<T>,
}

impl<T: Payload> Consumer<T> {
    fn new(channel: Channel<T>) -> Self {
        Self {
            channel,
            queue: Queue::new(),
        }
    }

    pub fn wait_pull_waker(&self) -> crate::queue::Waker<T> {
        self.queue.waker()
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
        self.channel
            .queues_write()
            .retain(|queue| !(queue == &self.queue));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_payload::*;
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    #[test]
    fn test_instantiate_producers() {
        let channel = Channel::<TestPayload>::new();

        let n: usize = 10;
        let mut v = Vec::<Producer<TestPayload>>::new();
        for _ in 0..n {
            v.push(channel.new_producer());
        }
    }

    #[test]
    fn test_instantiate_consumers() {
        let channel = Channel::<TestPayload>::new();

        let n: usize = 10;
        let mut v = Vec::<Consumer<TestPayload>>::new();
        for _ in 0..n {
            v.push(channel.new_consumer());
        }
        assert_eq!(v.len(), n);
        assert_eq!(v.first().unwrap().channel.queues_read().len(), n);
    }

    #[test]
    fn test_drop_consumer() {
        let channel = Channel::<TestPayload>::new();

        let n: usize = 10;
        let mut v = Vec::<Consumer<TestPayload>>::new();
        for i in 0..n {
            assert_eq!(channel.queues_read().len(), i);
            v.push(channel.new_consumer());
            channel.new_producer().push(TestPayload::new(i)); // so that we can identify consumers
        }

        assert_eq!(channel.queues_read().len(), n);

        for i in 0..n {
            assert_eq!(v.pop().unwrap().queue.pull().len(), i + 1); // make sure that we're popping the right consumer
            assert_eq!(channel.queues_read().len(), n - i - 1);
        }
    }

    #[test]
    fn test_consumer_api() {
        let channel = Channel::<TestPayload>::new();
        let consumer = channel.new_consumer();

        assert_eq!(consumer.queue.len(), 0);
        assert_eq!(consumer.pull().len(), 0);

        consumer
            .queue
            .push(Message::new(Instant::now(), TestPayload::default()));
        assert_eq!(consumer.queue.len(), 1);
        consumer
            .queue
            .push(Message::new(Instant::now(), TestPayload::default()));
        assert_eq!(consumer.queue.len(), 2);
        assert_eq!(consumer.pull().len(), 2);
        assert_eq!(consumer.queue.len(), 0);

        consumer
            .queue
            .push(Message::new(Instant::now(), TestPayload::default()));
        assert_eq!(consumer.queue.len(), 1);
        assert_eq!(consumer.wait_pull().len(), 1);
        assert_eq!(consumer.queue.len(), 0);

        let wait_time = Duration::from_millis(200);
        let queue_for_thread = consumer.queue.clone();
        let msg = Message::new(Instant::now(), TestPayload::default());
        let now = Instant::now();
        let _ = std::thread::spawn(move || {
            sleep(wait_time);
            queue_for_thread.push(msg)
        });
        assert_eq!(consumer.wait_pull().len(), 1);
        let elapsed = now.elapsed();
        assert!(elapsed >= wait_time);
    }

    #[test]
    fn test_consumer_waker() {
        const WAIT_TIME: Duration = Duration::from_millis(200);

        let channel = Channel::<TestPayload>::new();
        let consumer = channel.new_consumer();

        let start_time = Instant::now();

        let waker = consumer.wait_pull_waker();

        let join_handle = std::thread::spawn(move || {
            sleep(WAIT_TIME);
            waker.wake_up()
        });

        let v = consumer.wait_pull();
        let elapsed = start_time.elapsed();

        join_handle.join().unwrap();

        assert!(elapsed >= WAIT_TIME);
        assert!(v.is_empty());

    }
}
