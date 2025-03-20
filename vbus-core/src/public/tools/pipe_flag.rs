use os_pipe::{PipeReader, PipeWriter};
use std::io::Write;
use std::os::fd::{AsRawFd, RawFd};

pub fn pipe_flag() -> (PipeFlagReader, PipeFlagWriter) {
    let (reader, writer) = os_pipe::pipe().unwrap();
    (PipeFlagReader::new(reader), PipeFlagWriter::new(writer))
}

pub struct PipeFlagReader {
    reader: PipeReader,
}

impl PipeFlagReader {
    fn new(reader: PipeReader) -> Self {
        PipeFlagReader { reader }
    }
}

impl AsRawFd for PipeFlagReader {
    fn as_raw_fd(&self) -> RawFd {
        self.reader.as_raw_fd()
    }
}

pub struct PipeFlagWriter {
    writer: PipeWriter,
}

impl PipeFlagWriter {
    fn new(writer: PipeWriter) -> Self {
        PipeFlagWriter { writer }
    }

    pub fn raise(&mut self) {
        const STOP: [u8; 1] = [0x00];
        self.writer.write_all(STOP.as_slice()).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_pipe_flag() {
        let (reader, mut writer) = pipe_flag();

        let mut selector = selecting::Selector::new();
        selector.add_read(&reader.as_raw_fd());

        let first_result = selector.select_timeout(Duration::ZERO).unwrap();
        assert_eq!(first_result.is_read(&reader.as_raw_fd()), false);

        writer.raise();
        let second_result = selector.select_timeout(Duration::ZERO).unwrap();
        assert_eq!(second_result.is_read(&reader.as_raw_fd()), true);
    }
}
