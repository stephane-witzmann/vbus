use crate::Payload;

#[derive(bincode::Encode, bincode::Decode, PartialEq, Debug)]
pub(crate) struct TestPayload {
    data: usize,
}

impl Payload for TestPayload {}

impl TestPayload {
    pub fn new(data: usize) -> Self {
        Self { data }
    }

    pub fn check(&self, value: usize) {
        assert_eq!(self.data, value);
    }

    pub fn value(&self) -> usize {
        self.data
    }
}

impl Default for TestPayload {
    fn default() -> Self {
        Self::new(0)
    }
}
