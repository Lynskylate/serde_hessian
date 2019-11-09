use std::io;


#[derive(Clone, PartialEq, Debug)]
pub enum ErrorCode{
    EofWhileParsing,
    IoError,
    UnknownType,
}
