use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum Error {
    /// I/O error.
    IoError(std::io::Error),
    /// Error occured when reading an object file header.
    GoblinError(goblin::error::Error),
    /// Unsupported object file format (only ELF is supported at the moment).
    UnsupportedObjectType(String),
    /// Invalid string (probe name, provider name or argument format).
    Utf8Error(FromUtf8Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}

impl From<goblin::error::Error> for Error {
    fn from(e: goblin::error::Error) -> Self {
        Error::GoblinError(e)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(e: FromUtf8Error) -> Self {
        Error::Utf8Error(e)
    }
}
