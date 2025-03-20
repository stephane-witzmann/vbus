use crate::Payload;
use crate::private::io::InputStream;
use crate::{Channel, Error};
use std::path::Path;
use std::thread::{JoinHandle, spawn};

pub struct Player<T: Payload> {
    _phantom: std::marker::PhantomData<T>,
    thread_join_handle: Option<JoinHandle<()>>, // Option -> we can own the handle in drop()
}

impl<T: Payload> Player<T> {
    pub(crate) fn new(channel: &Channel<T>, path: &Path) -> Result<Self, Error> {
        let file = std::fs::File::open(path)?;
        let mut stream = InputStream::<T>::new(Box::from(file))?;
        let channel_for_thread = channel.clone();

        let thread_join_handle = spawn(move || {
            loop {
                match stream.get() {
                    Ok(message) => channel_for_thread.push_message(message),
                    Err(_) => return,
                }
            }
        });

        Ok(Self {
            _phantom: std::marker::PhantomData,
            thread_join_handle: Some(thread_join_handle),
        })
    }
}

impl<T: Payload> Drop for Player<T> {
    fn drop(&mut self) {
        self.thread_join_handle.take().unwrap().join().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::Channel;
    use crate::private::test_tools::{TestDataSet, TestPayload, random_message_sequence};
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_player() {
        let data = random_message_sequence(100);
        let dataset = TestDataSet::new(&data).unwrap();

        let channel = Channel::<TestPayload>::new();
        let consumer = channel.new_consumer();
        let _player = channel.new_player(&dataset).unwrap();

        sleep(Duration::from_secs(1)); // should be more than enough
        let buffer = consumer.pull();

        assert_eq!(buffer.len(), data.len());

        for i in 0..data.len() {
            assert_eq!(
                buffer[i].get_payload().value(),
                data[i].get_payload().value()
            );
        }
    }
}
