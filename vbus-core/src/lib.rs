#![allow(dead_code)] // TODO remove this at some point

/*
 * Modules
 */

mod channel;
mod error;
mod flag;
mod io;
mod message;
mod processor;
mod queue;
mod storage;

/*
 * Public interface
 */

pub use channel::{Channel, Consumer, Producer};
pub use error::Error;
pub use message::Payload;

/*
 * Modules for testing
 */

#[cfg(test)]
mod temp_file;
#[cfg(test)]
mod test_payload;

#[cfg(test)]
mod tests {
    use crate::test_payload::TestPayload;
    use crate::*;
    use std::time::Instant;

    #[test]
    fn test_global() {
        let channel: Channel<TestPayload> = Channel::<TestPayload>::new();
        let _: Channel<TestPayload> = channel.clone();

        let producer: Producer<TestPayload> = channel.new_producer();
        producer.push(TestPayload::default());
        let _: &Channel<TestPayload> = producer.channel();

        let consumer: Consumer<TestPayload> = channel.new_consumer();
        let _: &Channel<TestPayload> = consumer.channel();

        let vec_content = [123456789, 42, 0];

        for content in vec_content {
            let start_time = Instant::now();
            producer.push(TestPayload::new(content));

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
