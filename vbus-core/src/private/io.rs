use crate::{Error, Message, Payload};
use std::io::{Read, Write};
use std::mem::MaybeUninit;
use std::time::Instant;

const MAGIC: [u8; 4] = [b'V', b'B', b'U', b'S'];
const CURRENT_VERSION: u16 = 1;

struct ChunkHeader {
    size: usize,
    time_stamp: Instant,
}

impl ChunkHeader {
    fn new(size: usize, time_stamp: Instant) -> Self {
        Self { size, time_stamp }
    }
}

struct ChunkData(Vec<u8>, Instant);

struct StreamHeader {
    magic: [u8; 4],
    version: u16,
}

impl Default for StreamHeader {
    fn default() -> Self {
        Self {
            magic: MAGIC,
            version: CURRENT_VERSION,
        }
    }
}

pub(crate) struct OutputStream<T: Payload> {
    write: Box<dyn Write + Send>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Payload> OutputStream<T> {
    pub fn new(write: Box<dyn Write + Send>) -> Result<Self, Error> {
        let mut stream = Self {
            write,
            _phantom: Default::default(),
        };

        stream.write_any(&StreamHeader::default())?;
        stream.append_string(T::format_name())?;

        Ok(stream)
    }

    pub fn append(&mut self, message: &Message<T>) -> Result<(), Error> {
        let encoded = bincode::encode_to_vec(message.get_payload(), bincode::config::standard())?;
        let chunk_data = ChunkData(encoded, message.get_type_stamp());
        self.append_bytes(chunk_data)?;
        Ok(())
    }

    fn append_string(&mut self, string: &str) -> Result<(), Error> {
        let chunk_data = ChunkData(string.as_bytes().to_vec(), Instant::now());
        self.append_bytes(chunk_data)
    }

    fn append_bytes(&mut self, chunk_data: ChunkData) -> Result<(), Error> {
        let header = ChunkHeader::new(chunk_data.0.len(), chunk_data.1);
        self.write_any(&header)?;
        self.write_bytes(chunk_data.0.as_slice())?;
        Ok(())
    }

    fn write_any<U: Sized>(&mut self, data: &U) -> Result<(), Error> {
        let bytes = unsafe { any_as_u8_slice(data) };
        self.write_bytes(bytes)?;
        Ok(())
    }

    fn write_bytes(&mut self, data: &[u8]) -> Result<(), Error> {
        self.write.write_all(data)?;
        Ok(())
    }
}

pub(crate) struct InputStream<T: Payload> {
    read: Box<dyn Read + Send>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Payload> InputStream<T> {
    pub fn new(read: Box<dyn Read + Send>) -> Result<Self, Error> {
        let mut stream = Self {
            read,
            _phantom: Default::default(),
        };

        stream.get_and_check_header()?;
        let format = stream.get_string()?;

        if format != T::format_name() {
            return Err(Error::BadFormat(format));
        }

        Ok(stream)
    }

    pub fn get(&mut self) -> Result<Message<T>, Error> {
        let chunk_data = self.read_chunk()?;
        let (decoded, _): (T, usize) = bincode::decode_from_slice(chunk_data.0.as_slice(), bincode::config::standard())?;
        Ok(Message::new(chunk_data.1, decoded))
    }

    fn get_and_check_header(&mut self) -> Result<(), Error> {
        let header = self.read_stream_header()?;

        if !header.magic.eq(&MAGIC) {
            return Err(Error::BadHeader);
        }

        if !header.version.eq(&CURRENT_VERSION) {
            return Err(Error::BadVersion(header.version));
        }

        Ok(())
    }

    fn get_string(&mut self) -> Result<String, Error> {
        let chunk_data = self.read_chunk()?;
        Ok(String::from_utf8_lossy(chunk_data.0.as_slice()).to_string())
    }

    fn read_stream_header(&mut self) -> Result<StreamHeader, Error> {
        let mut buffer = MaybeUninit::<StreamHeader>::uninit();
        let slice = unsafe { any_as_u8_mut_slice(&mut buffer) };
        self.read.read_exact(slice)?;
        unsafe { Ok(buffer.assume_init()) }
    }

    fn read_chunk(&mut self) -> Result<ChunkData, Error> {
        let header = &self.read_chunk_header()?;
        self.read_chunk_data(header)
    }

    fn read_chunk_header(&mut self) -> Result<ChunkHeader, Error> {
        let mut buffer = MaybeUninit::<ChunkHeader>::uninit();

        // Handle regular Eof (= 0 byte read)
        let size = match self.read.read(unsafe { any_as_u8_mut_slice(&mut buffer) }) {
            Ok(x) => {
                if x == 0 {
                    return Err(Error::RegularEof);
                }
                x
            }
            Err(e) => return Err(Error::StdIo(e)),
        };

        // Get remaining bytes (if missing)
        if size < size_of::<ChunkHeader>() {
            let (_, slice) = unsafe { any_as_u8_mut_slice(&mut buffer) }.split_at_mut(size);
            self.read.read_exact(slice)?;
        }

        unsafe { Ok(buffer.assume_init()) }
    }

    fn read_chunk_data(&mut self, header: &ChunkHeader) -> Result<ChunkData, Error> {
        let mut buffer = Vec::with_capacity(header.size);

        // Avoid buffer initialization
        unsafe {
            buffer.spare_capacity_mut(); // not required by the compiler, but keeps Clippy quiet
            buffer.set_len(header.size);
        }

        self.read.read_exact(buffer.as_mut_slice())?;

        Ok(ChunkData(buffer, header.time_stamp))
    }
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts((p as *const T) as *const u8, size_of::<T>()) }
}

unsafe fn any_as_u8_mut_slice<T: Sized>(p: &mut T) -> &mut [u8] {
    unsafe { core::slice::from_raw_parts_mut((p as *mut T) as *mut u8, size_of::<T>()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::test_tools::{EmptyPayload, TestPayload};
    use std::time::Duration;

    #[test]
    fn test_default_stream_header() {
        let h = StreamHeader::default();
        assert_eq!(h.magic, MAGIC);
        assert_eq!(h.version, CURRENT_VERSION);
    }

    #[test]
    fn test_stream_header() {
        let (reader, writer) = os_pipe::pipe().unwrap();
        let _ostream = OutputStream::<TestPayload>::new(Box::new(writer)).unwrap();
        InputStream::<TestPayload>::new(Box::new(reader)).unwrap();
    }

    #[test]
    fn test_closed_before_stream_header() {
        let (reader, _) = os_pipe::pipe().unwrap();
        match InputStream::<TestPayload>::new(Box::new(reader)) {
            Err(Error::StdIo(_)) => {}
            Ok(_) => panic!("Shouldn't succeed"),
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_stream_header_error_magic() {
        let (reader, mut writer) = os_pipe::pipe().unwrap();
        let mut bad_header = StreamHeader::default();
        *bad_header.magic.last_mut().unwrap() = bad_header.magic.last().unwrap() - 1;
        assert_ne!(bad_header.magic, MAGIC);

        let slice = unsafe { any_as_u8_slice(&bad_header) };
        writer.write_all(slice).unwrap();

        match InputStream::<TestPayload>::new(Box::new(reader)) {
            Err(Error::BadHeader) => {}
            _ => panic!("Expected BadHeader error"),
        }
    }

    #[test]
    fn test_stream_header_error_version() {
        let (reader, mut writer) = os_pipe::pipe().unwrap();
        let mut bad_header = StreamHeader::default();
        bad_header.version += 1;
        assert_ne!(bad_header.version, CURRENT_VERSION);

        let slice = unsafe { any_as_u8_slice(&bad_header) };
        writer.write_all(slice).unwrap();

        match InputStream::<TestPayload>::new(Box::new(reader)) {
            Err(Error::BadVersion(v)) => {
                if v != bad_header.version {
                    panic!("Wrong version in BadVersion error: {}", v);
                }
            }
            _ => panic!("Expected BadVersion error"),
        }
    }

    #[test]
    fn test_stream_header_error_format() {
        assert_ne!(TestPayload::format_name(), EmptyPayload::format_name());

        let (reader, writer) = os_pipe::pipe().unwrap();
        let _ostream = OutputStream::<EmptyPayload>::new(Box::new(writer)).unwrap();
        match InputStream::<TestPayload>::new(Box::new(reader)) {
            Err(Error::BadFormat(format)) => {
                if format != EmptyPayload::format_name() {
                    panic!("Wrong format in BadFormat error: {}", format);
                }
            }
            _ => panic!("Expected BadFormat error"),
        }
    }

    #[test]
    fn test_stream_close() {
        let (reader, writer) = os_pipe::pipe().unwrap();
        OutputStream::<TestPayload>::new(Box::new(writer)).unwrap();
        let mut istream = InputStream::<TestPayload>::new(Box::new(reader)).unwrap();
        match istream.get() {
            Err(Error::RegularEof) => {}
            _ => panic!("Expected RegularEof error"),
        }
    }

    #[test]
    fn test_stream_transfer() {
        let (reader, writer) = os_pipe::pipe().unwrap();
        let mut ostream = OutputStream::<TestPayload>::new(Box::new(writer)).unwrap();
        let mut istream = InputStream::<TestPayload>::new(Box::new(reader)).unwrap();

        let now = Instant::now();

        for i in 0..100usize {
            let time_stamp = now + Duration::from_secs(i as u64);
            let m = Message::new(time_stamp, TestPayload::new(i));
            ostream.append(&m).unwrap();
            let read = istream.get().unwrap();
            assert_eq!(read.get_type_stamp(), time_stamp);
            read.get_payload().check(i);
        }

        drop(ostream);

        // Check eof
        match istream.get() {
            Err(Error::RegularEof) => {}
            _ => {
                panic!("Expected RegularEof error");
            }
        }
    }

    #[test]
    fn test_stream_slow() {
        const PAYLOAD_VALUE: usize = 42;
        const SLEEP_TIME: Duration = Duration::from_millis(100);

        let (reader, writer) = os_pipe::pipe().unwrap();
        let mut ostream = OutputStream::<TestPayload>::new(Box::new(writer)).unwrap();
        let mut istream = InputStream::<TestPayload>::new(Box::new(reader)).unwrap();

        // Have the reader wait for some data
        let handle = std::thread::spawn(move || {
            istream.get().unwrap().get_payload().check(PAYLOAD_VALUE);
        });

        let encoded = bincode::encode_to_vec(&TestPayload::new(PAYLOAD_VALUE), bincode::config::standard()).unwrap();
        let header = ChunkHeader {
            size: encoded.len(),
            time_stamp: Instant::now(),
        };
        let header_slice = unsafe { any_as_u8_slice(&header) };

        // Write header one byte at a time
        for c in header_slice {
            std::thread::sleep(SLEEP_TIME);
            ostream.write_any(c).unwrap();
        }

        // Write payload
        ostream.write_bytes(encoded.as_slice()).unwrap();

        handle.join().unwrap();
    }
}
