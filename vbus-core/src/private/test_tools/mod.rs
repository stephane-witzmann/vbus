mod empty_payload;
mod temp_file;
mod test_dataset;
mod test_payload;

pub(crate) use empty_payload::EmptyPayload;
pub(crate) use temp_file::TempFile;
pub(crate) use test_dataset::TestDataSet;
pub(crate) use test_payload::TestPayload;

use crate::Message;
use rand::{Rng, rng};
use std::time::{Duration, Instant};

pub(crate) fn random_message_sequence(n: usize) -> Vec<Message<TestPayload>> {
    let mut output = Vec::<Message<TestPayload>>::new();

    let origin = Instant::now() - Duration::from_secs(3600 * 24 * 7); // start a week ago
    let mut offset = 0;

    for _ in 0..=n {
        offset += rng().random_range(0..100);
        let ts = origin + Duration::from_millis(offset);
        let payload = TestPayload::new(rng().random_range(usize::MIN..=usize::MAX));
        output.push(Message::new(ts, payload));
    }

    output
}
