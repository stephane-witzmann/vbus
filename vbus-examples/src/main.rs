use std::time::Instant;
use vbus_core::Channel;
use vbus_core::Payload;

pub struct TestPayload {
}

impl TestPayload {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl Payload for TestPayload {
    fn format_name() -> &'static str {
        "TestPayload"
    }

    fn to_binary(&self) -> Vec<u8> {
        todo!()
    }

    fn from_binary(_: &Vec<u8>) -> Option<Self> {
        todo!()
    }
}

fn main() {
    let channel = Channel::<TestPayload>::new(Instant::now());
    let p = channel.new_producer();
    let c = channel.new_consumer();
    p.push(TestPayload::new());
    assert_eq!(c.pull().len(), 1);
}
