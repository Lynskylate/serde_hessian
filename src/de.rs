use std::io;
use std::io::{Read, BufReader};
use std::collections::BTreeMap;

use super::value::Value;
use std::result;
use crate::constant::ByteCodecType;
use crate::error::{Result, Error, ErrorCode};
use crate::error::Error::{SyntaxError, IoError};

type MemoId = u32;


pub struct Deserializer<R: Read> {
    buffer: BufReader<R>,
    pos: usize,
    references: BTreeMap<MemoId, Value>,
}

impl<R: Read> Deserializer<R> {
    pub fn new(rd: R) -> Deserializer<R> {
        Deserializer {
            buffer: BufReader::new(rd),
            pos: 0,
            references: BTreeMap::new(),
        }
    }

    fn error<T>(&self, err: ErrorCode)-> Result<T>{
        Err(SyntaxError(err))
    }


    #[inline]
    fn read_byte(&mut self) -> Result<u8> {
        let mut b = [0];
        match self.buffer.read(&mut b) {
            Ok(1) => {
                self.pos += 1;
                Ok(b[0])
            },
            Ok(_) => self.error(ErrorCode::EofWhileParsing),
            Err(e) => Err(IoError(e))
        }
    }

    #[inline]
    fn read_bytes(&mut self, n: usize) -> Result<Vec<u8>>{
        let mut buf = Vec::new();
        match self.buffer.by_ref().take(n as u64).read_to_end(&mut buf){
            Ok(m) if m == n => { self.pos += n; Ok(buf)}
            Ok(_) => self.error(ErrorCode::EofWhileParsing),
            Err(e) => Err(Error::IoError(e)),
        }
    }

    fn read_value(&mut self) -> Result<Value> {
        let v = self.read_byte()?;
        match ByteCodecType::from(v){
            ByteCodecType::ShortBinary(size) => {
                let b = self.read_bytes(size)?;
                Ok(Value::Bytes(b))
            }
            _ => self.error(ErrorCode::UnknownType),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_short_binary() {

    }
}
