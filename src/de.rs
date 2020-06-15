use std::collections::BTreeMap;
use std::io::{Cursor, Read};

use byteorder::{BigEndian, ReadBytesExt};

use super::constant::{Binary, ByteCodecType, Integer};
use super::error::Error::SyntaxError;
use super::error::{ErrorCode, Result};
use super::value::Value;

type MemoId = u32;

pub struct Deserializer<R: AsRef<[u8]>> {
    buffer: Cursor<R>,
    references: BTreeMap<MemoId, Value>,
}

impl<R: AsRef<[u8]>> Deserializer<R> {
    pub fn new(rd: R) -> Deserializer<R> {
        Deserializer {
            buffer: Cursor::new(rd),
            references: BTreeMap::new(),
        }
    }

    fn error<T>(&self, err: ErrorCode) -> Result<T> {
        Err(SyntaxError(err))
    }

    #[inline]
    fn read_byte(&mut self) -> Result<u8> {
        Ok(self.buffer.read_u8()?)
    }

    #[inline]
    fn read_bytes(&mut self, n: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0; n];
        self.buffer.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_long_binary(&mut self, tag: u8) -> Result<Value> {
        let mut buf = Vec::new();
        let mut tag = tag;
        // Get non-final chunk starts with 'A'
        while tag == b'A' {
            let length = self.buffer.read_i16::<BigEndian>()? as usize;
            buf.extend_from_slice(&self.read_bytes(length)?);
            tag = self.read_byte()?;
        }

        // FIXME: refactor duplicated code with read_binary
        match tag {
            b'B' => {
                // Get the last chunk starts with 'B'
                let length = self.buffer.read_i16::<BigEndian>()? as usize;
                buf.extend_from_slice(&self.read_bytes(length)?);
            }
            0x20..=0x2f => buf.extend_from_slice(&self.read_bytes((tag - 0x20) as usize)?),
            0x34..=0x37 => {
                let second_byte = self.read_byte()?;
                let v = self.read_bytes(i16::from_be_bytes([tag - 0x34, second_byte]) as usize)?;
                buf.extend_from_slice(&v);
            }
            _ => { /* TODO: error */ }
        }
        Ok(Value::Bytes(buf))
    }

    fn read_binary(&mut self, bin: Binary) -> Result<Value> {
        match bin {
            Binary::ShortBinary(b) => Ok(Value::Bytes(self.read_bytes((b - 0x20) as usize)?)),
            Binary::TwoOctetBinary(b) => {
                let second_byte = self.read_byte()?;
                let v = self.read_bytes(i16::from_be_bytes([b - 0x34, second_byte]) as usize)?;
                Ok(Value::Bytes(v))
            }
            Binary::LongBinary(b) => self.read_long_binary(b),
        }
    }

    fn read_int(&mut self, i: Integer) -> Result<Value> {
        match i {
            Integer::DirectInt(b) => Ok(Value::Int(b as i32 - 0x90)),
            Integer::ByteInt(b) => {
                let b2 = self.read_byte()?;
                Ok(Value::Int(
                    i16::from_be_bytes([b.overflowing_sub(0xc8).0, b2]) as i32,
                ))
            }
            Integer::ShortInt(b) => {
                let bs = self.read_bytes(2)?;
                //TODO: Optimize the code style
                Ok(Value::Int(
                    i32::from_be_bytes([b.overflowing_sub(0xd4).0, bs[0], bs[1], 0x00]) >> 8,
                ))
            }
            Integer::NormalInt => {
                let val = self.buffer.read_i32::<BigEndian>()?;
                Ok(Value::Int(val))
            }
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

pub fn value_from_slice(v: &[u8]) -> Result<Value> {
    let mut de = Deserializer::new(v);
    let value = de.read_value()?;
    Ok(value)
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
