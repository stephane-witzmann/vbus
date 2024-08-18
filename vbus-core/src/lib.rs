#![allow(dead_code)]

mod payload;
mod channel;
mod message;
mod queue;
mod test_payload;

pub use channel::{Channel, Consumer, Producer};
pub use payload::Payload;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    use crate::test_payload::TestPayload;

    #[test]
    fn test_global() {
        let t0 = Instant::now();
        let channel: Channel<TestPayload> = Channel::<TestPayload>::new(t0);
        let _: Channel<TestPayload> = channel.clone();
        let _: Duration = channel.now();

        let producer: Producer<TestPayload> = channel.new_producer();
        producer.push(TestPayload::new(0));
        let _: &Channel<TestPayload> = producer.channel();

        let consumer: Consumer<TestPayload> = channel.new_consumer();
        let _: &Channel<TestPayload> = consumer.channel();

        let content: usize = 123456789;
        producer.push(TestPayload::new(content));
        consumer.pull().first().unwrap().get_payload().check(content);

        let other_content: usize = 42;
        producer.push(TestPayload::new(other_content));
        consumer.wait_pull().first().unwrap().get_payload().check(other_content);
    }
}
