use std::result;
use std::string::FromUtf8Error;
use std::{fmt, io};

#[derive(Clone, PartialEq, Debug)]
pub enum ErrorKind {
    UnknownType,
    UnexpectedType(String),
    OutOfTypeRefRange(usize),
    OutOfDefinitionRange(usize),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ErrorKind::*;

        match self {
            UnknownType => write!(f, "unknown type"),
            UnexpectedType(typ) => write!(f, "unexpected type {}", typ),
            OutOfTypeRefRange(index) => write!(f, "out of type ref range: {}", index),
            OutOfDefinitionRange(index) => write!(f, "out of type definition range: {}", index),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    SyntaxError(ErrorKind),
    IoError(io::Error),
    FromUtf8Error(FromUtf8Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::SyntaxError(err) => write!(f, "syntax error: {}", err),
            Error::IoError(err) => err.fmt(f),
            Error::FromUtf8Error(err) => err.fmt(f),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::IoError(error)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(error: FromUtf8Error) -> Self {
        Error::FromUtf8Error(error)
    }
}

pub type Result<T> = result::Result<T, Error>;
