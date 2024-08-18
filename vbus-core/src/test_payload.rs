use crate::payload::Payload;

pub struct TestPayload {
    data: usize,
}

impl TestPayload {
    pub fn new(data: usize) -> Self {
        Self {
            data,
        }
    }

    pub fn check(&self, value: usize) {
        assert_eq!(self.data, value);
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