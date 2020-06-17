use crate::error::Error::IoError;
use std::io;
use std::result;

#[derive(Clone, PartialEq, Debug)]
pub enum ErrorKind {
    UnExpectError(String),
    UnknownType,
}

#[derive(Debug)]
pub enum Error {
    SyntaxError(ErrorKind),
    IoError(io::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        IoError(error)
    }
}

pub type Result<T> = result::Result<T, Error>;
