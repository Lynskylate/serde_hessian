use std::collections::BTreeMap;
use std::io::{BufReader, Read};

use super::constant::{Binary, ByteCodecType};
use super::error::Error::{IoError, SyntaxError};
use super::error::{Error, ErrorCode, Result};
use super::value::Value;
use crate::value::Value::Null;

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

    fn error<T>(&self, err: ErrorCode) -> Result<T> {
        Err(SyntaxError(err))
    }

    #[inline]
    fn read_byte(&mut self) -> Result<u8> {
        let mut b = [0];
        match self.buffer.read(&mut b) {
            Ok(1) => {
                self.pos += 1;
                Ok(b[0])
            }
            Ok(_) => self.error(ErrorCode::EofWhileParsing),
            Err(e) => Err(IoError(e)),
        }
    }

    #[inline]
    fn read_bytes(&mut self, n: usize) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        match self.buffer.by_ref().take(n as u64).read_to_end(&mut buf) {
            Ok(m) if m == n => {
                self.pos += n;
                Ok(buf)
            }
            Ok(_) => self.error(ErrorCode::EofWhileParsing),
            Err(e) => Err(Error::IoError(e)),
        }
    }

    fn read_long_binary(&mut self, tag: u8) -> Result<Value> {
        //Todo: Implement for long binary
        Ok(Null)
    }

    pub fn read_value(&mut self) -> Result<Value> {
        let v = self.read_byte()?;
        match ByteCodecType::from(v) {
            ByteCodecType::Binary(bin) => match bin {
                Binary::ShortBinary(b) => self
                    .read_bytes((b - 0x20) as usize)
                    .and_then(|b| Ok(Value::Bytes(b)))
                    .map_err(From::from),
                Binary::TwoOctetBinary(b) => self
                    .read_byte()
                    .and_then(|second_byte| {
                        self.read_bytes(i16::from_be_bytes([b - 0x34, second_byte]) as usize)
                    })
                    .and_then(|v| Ok(Value::Bytes(v)))
                    .map_err(From::from),
                Binary::LongBinary(b) => self.read_long_binary(b).map_err(From::from),
            },
            ByteCodecType::True => Ok(Value::Bool(true)),
            ByteCodecType::False => Ok(Value::Bool(false)),
            ByteCodecType::Null => Ok(Value::Null),
            _ => self.error(ErrorCode::UnknownType),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Deserializer;
    use crate::value::Value;
    use std::io::BufReader;

    fn test_decode_ok(rdr: &[u8], target: Value) {
        let mut de = Deserializer::new(rdr);
        let value = de.read_value();
        assert!(value.is_ok());
        assert_eq!(value.ok().unwrap(), target);
    }
    #[test]
    fn test_short_binary() {}

    #[test]
    fn test_boolean() {
        test_decode_ok(&[b'T'], Value::Bool(true));
        test_decode_ok(&[b'F'], Value::Bool(false));
    }

    #[test]
    fn test_null() {
        test_decode_ok(&[b'N'], Value::Null);
    }
}
