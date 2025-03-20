use crate::private::queue::Queue;
use crate::{Channel, Message, Payload};

pub(crate) struct Consumer<T: Payload> {
    channel: Channel<T>,
    queue: Queue<T>,
}

impl<T: Payload> Consumer<T> {
    pub fn new(channel: &Channel<T>) -> Self {
        let queue = Queue::<T>::new();
        channel.add_queue(&queue);

        Self {
            channel: channel.clone(),
            queue,
        }
    }

    pub fn wait_pull_waker(&self) -> crate::private::queue::Waker<T> {
        self.queue.waker()
    }

    pub fn wait_pull(&self) -> Vec<Message<T>> {
        self.queue.wait_pull()
    }

    #[cfg(test)]
    pub(crate) fn pull(&self) -> Vec<Message<T>> {
        self.queue.pull()
    }

    #[cfg(test)]
    pub(crate) fn channel(&self) -> &Channel<T> {
        &self.channel
    }
}

impl<T: Payload> Drop for Consumer<T> {
    fn drop(&mut self) {
        self.channel.remove_queue(&self.queue);
    }
}

#[cfg(test)]
mod tests {
    use crate::private::Consumer;
    use crate::private::test_tools::TestPayload;
    use crate::{Channel, Message};
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    #[test]
    fn test_instantiate_consumers() {
        let channel = Channel::<TestPayload>::new();

        let n: usize = 10;
        let mut v = Vec::<Consumer<TestPayload>>::new();
        for _ in 0..n {
            v.push(channel.new_consumer());
        }
        assert_eq!(v.len(), n);
        assert_eq!(v.first().unwrap().channel.queues_len(), n);
    }

    #[test]
    fn test_drop_consumer() {
        let channel = Channel::<TestPayload>::new();

        let n: usize = 10;
        let mut v = Vec::<Consumer<TestPayload>>::new();
        for i in 0..n {
            assert_eq!(channel.queues_len(), i);
            v.push(channel.new_consumer());
            channel.push(TestPayload::new(i)); // so that we can identify consumers
        }

        assert_eq!(channel.queues_len(), n);

        for i in 0..n {
            assert_eq!(v.pop().unwrap().queue.pull().len(), i + 1); // make sure that we're popping the right consumer
            assert_eq!(channel.queues_len(), n - i - 1);
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
