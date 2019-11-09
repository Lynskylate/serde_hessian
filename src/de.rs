use std::io;
use std::io::{Read, BufReader};
use std::collections::BTreeMap;

use super::value::Value;
use super::error::ErrorCode;
use std::fmt::Error;
use std::result;
use crate::constant::ByteCodecType;

type MemoId = u32;

pub type Result<T> = result::Result<T, ErrorCode>;

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


    #[inline]
    fn read_byte(&mut self) -> Result<u8> {
        let mut b = [0];
        match self.buffer.read(&mut b) {
            Ok(1) => {
                self.pos += 1;
                Ok(b[0])
            },
            Ok(_) => Err(ErrorCode::EofWhileParsing),
            Err(e) => Err(ErrorCode::IoError),
        }
    }

    #[inline]
    fn read_bytes(&mut self, n: usize) -> Result<Vec<u8>>{
        let mut buf = Vec::new();
        match self.buffer.by_ref().take(n as u64).read_to_end(&mut buf){
            Ok(m) if m == n => { self.pos += n; Ok(buf)}
            Ok(_) => Err(ErrorCode::EofWhileParsing),
            Err(e) => Err(ErrorCode::IoError),
        }
    }

    fn read_value(&mut self) -> Result<Value> {
        let v = self.read_byte()?;
        match ByteCodecType::from(v){
            ByteCodecType::ShortBinary(size) => {
                let b = self.read_bytes(size)?;
                Ok(Value::Bytes(b))
            }
            _ => Err(ErrorCode::UnknownType),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_short_binary() {

    }
}
