#![allow(dead_code)]

use vbus_core::{Channel, Payload};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TestPayload {}

impl TestPayload {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for TestPayload {
    fn default() -> Self {
        Self::new()
    }
}

impl Payload for TestPayload {}

fn main() {
    let channel = Channel::<TestPayload>::new();
    let p = channel.new_producer();
    let c = channel.new_consumer();
    p.push(TestPayload::new());
    assert_eq!(c.pull().len(), 1);
}
