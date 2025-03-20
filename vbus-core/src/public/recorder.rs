use crate::ThreadedConsumer;
use crate::private::io::OutputStream;
use crate::{Channel, Error, Payload};
use std::fs::File;
use std::path::Path;

pub struct Recorder<T: Payload> {
    _tc: ThreadedConsumer<T>,
}

impl<T: Payload> Recorder<T> {
    pub(crate) fn new(channel: &Channel<T>, path: &Path) -> Result<Self, Error> {
        let file = Box::new(File::create(path)?);
        let mut ostream = OutputStream::<T>::new(file)?;

        let tc = ThreadedConsumer::new(channel.new_consumer(), move |messages| {
            for m in messages {
                ostream.append(&m).unwrap(); // TODO handle error here
            }
        });

        Ok(Self { _tc: tc })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Message;
    use crate::private::io::InputStream;
    use crate::private::test_tools::TempFile;
    use crate::private::test_tools::TestPayload;
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    #[test]
    fn test_recorder() {
        let temp_file = TempFile::new().unwrap();
        let now = Instant::now();
        let reference = [
            Message::new(now, TestPayload::new(123456789)),
            Message::new(now + Duration::from_nanos(1), TestPayload::new(42)),
            Message::new(now + Duration::from_secs(2), TestPayload::new(42)),
        ];

        record(&temp_file, &reference).unwrap();
        let actual = read(&temp_file).unwrap();

        let ref_ts = reference
            .iter()
            .map(|m| m.get_type_stamp())
            .collect::<Vec<_>>();
        let actual_ts = actual
            .iter()
            .map(|m| m.get_type_stamp())
            .collect::<Vec<_>>();
        assert_eq!(actual_ts, ref_ts);

        let ref_payload = reference
            .iter()
            .map(|m| m.get_payload())
            .collect::<Vec<_>>();
        let actual_payload = actual.iter().map(|m| m.get_payload()).collect::<Vec<_>>();
        assert_eq!(actual_payload, ref_payload);
    }

    fn record(path: &Path, data: &[Message<TestPayload>]) -> Result<(), Error> {
        let channel: Channel<TestPayload> = Channel::<TestPayload>::new();
        let _recorder = Recorder::new(&channel, path)?;

        data.iter().for_each(|x| channel.push_message(x.clone()));

        sleep(Duration::from_millis(500));

        Ok(())
    }

    fn read(path: &Path) -> Result<Vec<Message<TestPayload>>, Error> {
        let file = File::open(path)?;
        let mut istream = InputStream::<TestPayload>::new(Box::new(file))?;
        let mut v = Vec::<Message<TestPayload>>::new();

        loop {
            match istream.get() {
                Ok(message) => v.push(message),
                Err(Error::RegularEof) => break,
                Err(e) => return Err(e),
            }
        }

        Ok(v)
    }
}
