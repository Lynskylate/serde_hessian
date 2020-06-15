use std::collections::BTreeMap;
use std::io::{BufReader, Read};

use super::constant::{Binary, ByteCodecType, Integer};
use super::error::Error::{IoError, SyntaxError};
use super::error::{Error, ErrorCode, Result};
use super::value::Value;
use crate::value::Value::Null;
use std::convert::TryInto;

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

    fn read_binary(&mut self, bin: Binary) -> Result<Value> {
        match bin {
            Binary::ShortBinary(b) => self
                .read_bytes((b - 0x20) as usize)
                .and_then(|b| Ok(Value::Bytes(b))),
            Binary::TwoOctetBinary(b) => self
                .read_byte()
                .and_then(|second_byte| {
                    self.read_bytes(i16::from_be_bytes([b - 0x34, second_byte]) as usize)
                })
                .and_then(|v| Ok(Value::Bytes(v))),
            Binary::LongBinary(b) => self.read_long_binary(b),
        }
    }

    fn read_int(&mut self, i: Integer) -> Result<Value> {
        match i {
            Integer::DirectInt(b) => Ok(Value::Int(b as i32 - 0x90)),
            Integer::ByteInt(b) => self
                .read_byte()
                .and_then(|b2| {
                    Ok(Value::Int(
                        i16::from_be_bytes([b.overflowing_sub(0xc8).0, b2]) as i32,
                    ))
                }),
            Integer::ShortInt(b) => self
                .read_bytes(2)
                //TODO: Optimize the code style
                .and_then(|bs| {
                    Ok(Value::Int(
                        i32::from_be_bytes([b.overflowing_sub(0xd4).0, bs[0], bs[1], 0x00]) >> 8,
                    ))
                }),
            Integer::NormalInt => self
                .read_bytes(4)
                .and_then(|bs| {
                    Ok(Value::Int(i32::from_be_bytes(
                        bs.as_slice().try_into().unwrap(),
                    )))
                }),
        }
    }

    pub fn read_value(&mut self) -> Result<Value> {
        let v = self.read_byte()?;
        match ByteCodecType::from(v) {
            ByteCodecType::Int(i) => self.read_int(i),
            ByteCodecType::Binary(bin) => self.read_binary(bin),
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

    fn test_decode_ok(rdr: &[u8], target: Value) {
        let mut de = Deserializer::new(rdr);
        let value = de.read_value();
        assert!(value.is_ok());
        assert_eq!(value.ok().unwrap(), target);
    }

    #[test]
    fn test_decode_int() {
        test_decode_ok(&[b'I', 0x00, 0x00, 0x00, 0x00], Value::Int(0));
        test_decode_ok(&[0x90u8], Value::Int(0));
        test_decode_ok(&[0x80u8], Value::Int(-16));
        test_decode_ok(&[0xbfu8], Value::Int(47));
        test_decode_ok(&[0xc8u8, 0x30u8], Value::Int(48));

        test_decode_ok(&[0xc0, 0x00], Value::Int(-2048));
        test_decode_ok(&[0xc7, 0x00], Value::Int(-256));
        test_decode_ok(&[0xcf, 0xff], Value::Int(2047));

        test_decode_ok(&[0xd0, 0x00, 0x00], Value::Int(-262144));
        test_decode_ok(&[0xd7, 0xff, 0xff], Value::Int(262143));

        test_decode_ok(&[b'I', 0x00, 0x04, 0x00, 0x00], Value::Int(262144));
    }

    #[test]
    fn test_short_binary() {
        test_decode_ok(&[0x20], Value::Bytes(Vec::new()));
        test_decode_ok(&[0x23, 0x01, 0x02, 0x03], Value::Bytes(vec![1, 2, 3]));
    }

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
