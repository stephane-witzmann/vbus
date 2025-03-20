use crate::private::io::{InputStream, OutputStream};
use crate::private::test_tools::{TempFile, TestPayload, random_message_sequence};
use crate::{Error, Message, Payload};
use std::fs::File;
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::Path;

pub struct TestDataSet<T: Payload> {
    temp_file: TempFile,
    _phantom_data: PhantomData<T>,
}

impl<T: Payload> TestDataSet<T> {
    pub fn new(data: &Vec<Message<T>>) -> Result<Self, Error> {
        let temp_file = TempFile::new()?;

        write_all(&temp_file, data)?;

        Ok(TestDataSet {
            temp_file,
            _phantom_data: PhantomData,
        })
    }
}

impl<T: Payload> Deref for TestDataSet<T> {
    type Target = Path;
    fn deref(&self) -> &Self::Target {
        self.temp_file.path()
    }
}

fn write_all<T: Payload>(path: &Path, data: &Vec<Message<T>>) -> Result<(), Error> {
    let file = Box::new(File::create(path)?);
    let mut ostream = OutputStream::<T>::new(file)?;

    for message in data {
        ostream.append(message)?;
    }

    Ok(())
}

#[test]
fn test_dataset() {
    let messages = random_message_sequence(100);
    let dataset = TestDataSet::new(&messages).unwrap();
    let path: &Path = &dataset;
    let file = File::open(path).unwrap();
    let mut istream = InputStream::<TestPayload>::new(Box::new(file)).unwrap();

    for ref_message in messages.iter() {
        let read_message = istream.get().unwrap();
        assert_eq!(
            read_message.get_payload().value(),
            ref_message.get_payload().value()
        );
        assert_eq!(read_message.get_type_stamp(), ref_message.get_type_stamp());
    }

    let last = istream.get();
    if let Err(Error::RegularEof) = last {
        return; // regular exit, everything went well
    }

    if let Ok(_) = last {
        panic!("Got valid value when expecting EOF");
    }

    last.unwrap(); // will panic
    unreachable!()
}
