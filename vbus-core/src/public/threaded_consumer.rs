use crate::Payload;
use crate::private::Consumer;
use crate::private::queue::Waker;
use crate::tools::atomic_flag::*;
use std::thread::{JoinHandle, spawn};

pub struct ThreadedConsumer<T: Payload> {
    waker: Waker<T>,
    stopper: AtomicFlagWriter,
    thread_join_handle: Option<JoinHandle<()>>, // Option -> we can own the handle in drop()
}

impl<T: Payload> ThreadedConsumer<T> {
    pub(crate) fn new(
        consumer: Consumer<T>,
        mut process: impl FnMut(Vec<crate::Message<T>>) + Send + 'static,
    ) -> Self {
        let (flag_reader, flag_writer) = atomic_flag();

        let waker = consumer.wait_pull_waker();

        let thread_join_handle = spawn(move || {
            loop {
                let messages = consumer.wait_pull();

                if flag_reader.check() {
                    return;
                }

                process(messages);

                if flag_reader.check() {
                    return;
                }
            }
        });

        Self {
            waker,
            stopper: flag_writer,
            thread_join_handle: Some(thread_join_handle),
        }
    }
}

impl<T: Payload> Drop for ThreadedConsumer<T> {
    fn drop(&mut self) {
        self.stopper.raise();
        self.waker.wake_up();
        self.thread_join_handle.take().unwrap().join().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::ThreadedConsumer;
    use crate::Channel;
    use crate::private::test_tools::TestPayload;
    use std::sync::{Arc, Mutex};
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_threaded_consumer() {
        let channel = Channel::<TestPayload>::new();
        let reference = [123456789, 42, 0usize];
        let processed = Arc::new(Mutex::new(Vec::<usize>::new()));
        let processed_for_thread = processed.clone();

        let tc = ThreadedConsumer::new(channel.new_consumer(), move |input| {
            let mut output = processed_for_thread.lock().unwrap();
            input.iter().for_each(|message| {
                output.push(message.get_payload().value());
            });
        });

        reference
            .iter()
            .for_each(|x| channel.push(TestPayload::new(*x)));

        sleep(Duration::from_millis(500));
        drop(tc);
        sleep(Duration::from_millis(200));

        assert_eq!(processed.lock().unwrap().len(), reference.len());
        assert_eq!(processed.lock().unwrap().as_slice(), reference);
    }
}
