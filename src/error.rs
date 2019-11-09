use std::io;
use std::result;
use crate::error::Error::IoError;


#[derive(Clone, PartialEq, Debug)]
pub enum ErrorCode{
    EofWhileParsing,
    UnknownType,
}

pub enum Error{
    SyntaxError(ErrorCode),
    IoError(io::Error),
}

impl From<io::Error> for Error{
    fn from(error: io::Error) -> Error{
        IoError(error)
    }
}

pub type Result<T> = result::Result<T, Error>;
