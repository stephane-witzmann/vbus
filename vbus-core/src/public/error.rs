#[derive(Debug)]
pub enum Error {
    RegularEof,
    BadHeader,
    BadVersion(u16),
    BadFormat(String),
    StdIo(std::io::Error),
    BincodeDecode(bincode::error::DecodeError),
    BincodeEncode(bincode::error::EncodeError),
    NotImplemented,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::StdIo(value)
    }
}

impl From<bincode::error::DecodeError> for Error {
    fn from(value: bincode::error::DecodeError) -> Self {
        Error::BincodeDecode(value)
    }
}

impl From<bincode::error::EncodeError> for Error {
    fn from(value: bincode::error::EncodeError) -> Self {
        Error::BincodeEncode(value)
    }
}
