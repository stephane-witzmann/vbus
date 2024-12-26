#[derive(Debug)]
pub enum Error {
    RegularEof,
    BadHeader,
    BadVersion(u16),
    BadFormat(String),
    StdIo(std::io::Error),
    Bincode(bincode::Error),
    NotImplemented,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::StdIo(value)
    }
}

impl From<bincode::Error> for Error {
    fn from(value: bincode::Error) -> Self {
        Error::Bincode(value)
    }
}
