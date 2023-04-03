use hessian_rs::Error as HessianError;
use hessian_rs::ErrorKind;
use std::error::Error as StdError;

use std::string::FromUtf8Error;
use std::{fmt, io};

use serde::de::Error as DeError;
use serde::ser::Error as SerError;

#[derive(Debug)]
pub enum Error {
    SyntaxError(ErrorKind),
    IoError(io::Error),
    FromUtf8Error(FromUtf8Error),
    SerdeDesrializeError(String),
    SerdeSerializeError(String),
    UnSupportedRefType,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::SyntaxError(err) => write!(f, "syntax error: {}", err),
            Error::IoError(err) => err.fmt(f),
            Error::SerdeDesrializeError(err) => write!(f, "serde deserialize error: {}", err),
            Error::SerdeSerializeError(err) => write!(f, "serde serialize error: {}", err),
            Error::FromUtf8Error(err) => err.fmt(f),
            Error::UnSupportedRefType => write!(f, "unsupported ref type"),
        }
    }
}

impl From<HessianError> for Error {
    fn from(error: HessianError) -> Error {
        match error {
            HessianError::SyntaxError(err) => Error::SyntaxError(err),
            HessianError::IoError(err) => Error::IoError(err),
            HessianError::FromUtf8Error(err) => Error::FromUtf8Error(err),
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

impl SerError for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::SerdeSerializeError(msg.to_string())
    }
}

impl DeError for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::SerdeDesrializeError(msg.to_string())
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::SyntaxError(_) => None,
            Error::SerdeDesrializeError(_) => None,
            Error::SerdeSerializeError(_) => None,
            Error::IoError(err) => Some(err),
            Error::FromUtf8Error(err) => Some(err),
            Error::UnSupportedRefType => Some(self),
        }
    }
}
