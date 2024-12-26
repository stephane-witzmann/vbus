use crate::{io, Channel, Error, Payload};
use std::fs::File;
use std::path::Path;
use crate::processor::Processor;

pub struct Recorder<T: Payload> {
    processor: Processor<T>,
}

impl<T: Payload> Recorder<T> {
    fn new(channel: &Channel<T>, path: &Path) -> Result<Self, Error> {
        let file = Box::new(File::create(path)?);
        let mut ostream = io::OutputStream::<T>::new(file)?;

        let processor = Processor::new(channel.new_consumer(), move |messages| {
            for m in messages {
                ostream.append(m.get_payload()).unwrap(); // TODO handle error here
            }
        });

        Ok(Self {
            processor,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::thread::sleep;
    use std::time::Duration;
    use super::*;
    use crate::io::InputStream;
    use crate::temp_file::TempFile;
    use crate::test_payload::TestPayload;
    use crate::Producer;

    const CHANNEL_NAME: &str = "TestChannel";

    #[test]
    fn test_recorder() {
        let temp_file = TempFile::new("test_recorder_data.vbus").unwrap();
        let reference = [123456789, 42, 0usize];

        record(&temp_file, &reference).unwrap();
        let actual = read(&temp_file).unwrap();

        assert_eq!(actual.as_slice(), reference);
    }

    fn record(path: &Path, data: &[usize]) -> Result<(), Error> {
        let channel: Channel<TestPayload> = Channel::<TestPayload>::new();
        let _recorder = Recorder::new(&channel, path).unwrap();
        let producer: Producer<TestPayload> = channel.new_producer();

        data.iter()
            .for_each(|x| producer.push(TestPayload::new(*x)));

        sleep(Duration::from_millis(500));

        Ok(())
    }

    fn read(path: &Path) -> Result<Vec<usize>, Error> {
        let file = File::open(path)?;
        let mut istream = InputStream::<TestPayload>::new(Box::new(file))?;
        let mut v = Vec::<usize>::new();

        loop {
            match istream.get() {
                Ok(payload) => v.push(payload.value()),
                Err(Error::RegularEof) => break,
                Err(e) => return Err(e),
            }
        }

        Ok(v)
    }
}
