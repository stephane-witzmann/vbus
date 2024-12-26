use crate::flag::*;
use crate::{Consumer, Payload};
use std::thread::{spawn, JoinHandle};
use crate::queue::Waker;

pub struct Processor<T: Payload> {
    waker: Waker<T>,
    stopper: AtomicFlagWriter,
    thread_join_handle: Option<JoinHandle<()>>, // Option -> we can own the handle in drop()
}

impl<T: Payload> Processor<T> {
    pub fn new(consumer: Consumer<T>, mut process: impl FnMut(Vec<crate::message::Message<T>>) + Send + 'static) -> Self {
        let (flag_reader, flag_writer) = atomic_flag();

        let waker = consumer.wait_pull_waker();

        let thread_join_handle = spawn(move || loop {
            let messages = consumer.wait_pull();

            if flag_reader.check() {
                return;
            }

            process(messages);

            if flag_reader.check() {
                return;
            }
        });

        Self {
            waker,
            stopper: flag_writer,
            thread_join_handle: Some(thread_join_handle),
        }
    }
}

impl<T: Payload> Drop for Processor<T> {
    fn drop(&mut self) {
        self.stopper.raise();
        self.waker.wake_up();
        self.thread_join_handle.take().unwrap().join().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc,Mutex};
    use std::thread::sleep;
    use std::time::Duration;
    use super::Processor;
    use crate::{Channel, Producer};
    use crate::test_payload::TestPayload;

    #[test]
    fn test_processor() {
        let channel = Channel::<TestPayload>::default();
        let reference = [123456789, 42, 0usize];
        let processed = Arc::new(Mutex::new(Vec::<usize>::new()));
        let processed_for_thread = processed.clone();

        let processor = Processor::new(channel.new_consumer(), move |input| {
            let mut output = processed_for_thread.lock().unwrap();
            input.iter().for_each(|message| {
                output.push(message.get_payload().value());
            });
        });

        let producer: Producer<TestPayload> = channel.new_producer();
        reference.iter()
            .for_each(|x| producer.push(TestPayload::new(*x)));

        sleep(Duration::from_millis(500));
        drop(processor);
        sleep(Duration::from_millis(200));

        assert_eq!(processed.lock().unwrap().len(), reference.len());
        assert_eq!(processed.lock().unwrap().as_slice(), reference);
    }
}
