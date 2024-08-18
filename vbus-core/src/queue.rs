use crate::message::Message;
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use crate::payload::Payload;

pub struct Queue<T: Payload> {
    queue_data: Arc<QueueData<T>>,
}

struct QueueData<T: Payload> {
    messages: Mutex<VecDeque<Message<T>>>,
    condvar: Condvar,
}

impl<T: Payload> Queue<T> {
    pub fn new() -> Self {
        Self {
            queue_data: Arc::new(QueueData::new()),
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            queue_data: self.queue_data.clone(),
        }
    }

    pub fn push(&self, message: Message<T>) {
        self.queue_data.push(message);
    }

    pub fn pull(&self) -> Vec<Message<T>> {
        self.queue_data.pull()
    }

    pub fn wait_pull(&self) -> Vec<Message<T>> {
        self.queue_data.wait_pull()
    }

    pub fn wake_up(&self) {
        self.queue_data.wake_up();
    }

    pub fn len(&self) -> usize {
        self.queue_data.len()
    }
}

impl<T: Payload> PartialEq for Queue<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.queue_data, &other.queue_data)
    }
}

impl<T: Payload> QueueData<T> {
    fn new() -> Self {
        Self {
            messages: Mutex::new(VecDeque::<Message<T>>::new()),
            condvar: Condvar::new(),
        }
    }

    fn lock(&self) -> MutexGuard<'_, VecDeque<Message<T>>> {
        self.messages.lock().unwrap()
    }

    fn wake_up(&self) {
        let _lock = self.lock();
        self.condvar.notify_all();
    }

    fn push(&self, message: Message<T>) {
        let mut lock = self.lock();

        lock.push_back(message);
        self.condvar.notify_all();
    }

    fn pull(&self) -> Vec<Message<T>> {
        let mut v = Vec::<Message<T>>::new();

        {
            let mut lock = self.lock();
            Self::do_pull(&mut lock, &mut v);
        }

        v
    }

    fn wait_pull(&self) -> Vec<Message<T>> {
        let mut v = Vec::<Message<T>>::new();

        {
            let mut lock = self.lock();
            Self::do_pull(&mut lock, &mut v);
            if v.is_empty() {
                Self::do_pull(&mut self.condvar.wait(lock).unwrap(), &mut v);
            }
        }

        v
    }

    fn do_pull(lock: &mut MutexGuard<VecDeque<Message<T>>>, output: &mut Vec<Message<T>>) {
        while let Some(msg) = lock.pop_front() {
            output.push(msg);
        }
    }

    fn len(&self) -> usize {
        self.lock().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use crate::test_payload::TestPayload;

    fn push_many(queue: &Queue<TestPayload>, n: usize) {
        for i in 0..n {
            queue.push(Message::new(std::time::Duration::default(), TestPayload::new(i)));
        }
    }

    fn check_many(messages: &Vec<Message<TestPayload>>, n: usize) {
        assert_eq!(messages.len(), n);
        for i in 0..n {
            messages[i].get_payload().check(i);
        }
    }

    #[test]
    fn test_pull_and_wait_pull() {
        let queue = Queue::new();

        for n in 1..10 {
            assert_eq!(queue.pull().len(), 0);

            push_many(&queue, n);
            check_many(&queue.pull(), n);

            assert_eq!(queue.pull().len(), 0);

            push_many(&queue, n);
            check_many(&queue.wait_pull(), n);
        }

        assert_eq!(queue.pull().len(), 0);
    }

    #[test]
    fn test_wait_pull() {
        let queue = Queue::new();
        let wait_time = std::time::Duration::from_millis(200);
        let now = std::time::Instant::now();

        let queue_for_thread = queue.clone();
        let now_for_thread = now.clone();
        let join_handle = std::thread::spawn(move || {
            sleep(wait_time);
            queue_for_thread.push(Message::new(now_for_thread.elapsed(), TestPayload::new(0)));
        });

        let v = queue.wait_pull();
        let elapsed = now.elapsed();

        join_handle.join().unwrap();

        assert!(elapsed >= wait_time);
        assert!(elapsed >= v.first().unwrap().get_type_stamp());
    }

    #[test]
    fn test_eq() {
        let a = Queue::<TestPayload>::new();
        let a_clone = a.clone();
        assert!(a == a);
        assert!(a_clone == a_clone);
        assert!(a == a_clone);
        assert!(a_clone == a);

        let b = Queue::<TestPayload>::new();
        assert!(b == b);
        assert!(a != b);
        assert!(b != a);
        assert!(a_clone != b);
        assert!(b != a_clone);
    }
}
