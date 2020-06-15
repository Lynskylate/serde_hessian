use std::io;

use super::error::{Error, ErrorCode, Result};
use super::value::Value;

pub struct Serializer<W> {
    writer: W,
}

pub trait IdentifyLast: Iterator + Sized {
    fn identify_last(self) -> Iter<Self>;
}

impl<It> IdentifyLast for It
where
    It: Iterator,
{
    fn identify_last(mut self) -> Iter<Self> {
        let e = self.next();
        Iter {
            iter: self,
            buffer: e,
        }
    }
}

pub struct Iter<It>
where
    It: Iterator,
{
    iter: It,
    buffer: Option<It::Item>,
}

impl<It> Iterator for Iter<It>
where
    It: Iterator,
{
    type Item = (bool, It::Item);

    fn next(&mut self) -> Option<Self::Item> {
        match self.buffer.take() {
            None => None,
            Some(e) => match self.iter.next() {
                None => Some((true, e)),
                Some(f) => {
                    self.buffer = Some(f);
                    Some((false, e))
                }
            },
        }
    }
}

impl<W: io::Write> Serializer<W> {
    pub fn new(writer: W) -> Self {
        Serializer { writer }
    }

    pub fn into_inner(self) -> W {
        self.writer
    }

    pub fn serialize_value(&mut self, value: &Value) -> Result<()> {
        match *value {
            Value::Int(i) => self.serialize_int(i),
            Value::Bytes(ref b) => self.serialize_binary(b),
            Value::String(ref s) => self.serialize_string(s.as_str()),
            Value::Bool(b) => self.serialize_bool(b),
            Value::Null => self.serialize_null(),
            Value::Long(l) => self.serialize_long(l),
            _ => Err(Error::SyntaxError(ErrorCode::UnknownType)),
        }
    }

    fn serialize_null(&mut self) -> Result<()> {
        self.writer.write_all(&[b'N'])?;
        Ok(())
    }

    fn serialize_bool(&mut self, value: bool) -> Result<()> {
        let f = if value { b'T' } else { b'F' };
        self.writer.write_all(&[f])?;
        Ok(())
    }

    fn serialize_long(&mut self, v: i64) -> Result<()> {
        let bytes = match v {
            -8..=15 => vec![(0xd8 + v) as u8],
            -2048..=2047 => vec![(((v >> 8) + 0xf0) & 0xff) as u8, (v & 0xff) as u8],
            -262_144..=262_133 => vec![
                ((v >> 16) + 0x3c) as u8,
                ((v >> 8) & 0xff) as u8,
                (v & 0xff) as u8,
            ],
            _ if v >= i32::min_value() as i64 && v <= i32::max_value() as i64 => vec![
                0x59 as u8,
                (v >> 24 & 0xff) as u8,
                (v >> 16 & 0xff) as u8,
                (v >> 8 & 0xff) as u8,
                (v & 0xff) as u8,
            ],
            _ => Vec::from([&[b'L'], v.to_be_bytes().as_ref()].concat()),
        };
        self.writer.write_all(&bytes)?;
        Ok(())
    }

    fn serialize_int(&mut self, v: i32) -> Result<()> {
        let bytes = match v {
            -16..=47 => vec![(0x90 + v) as u8],
            -2048..=2047 => vec![(((v >> 8) & 0xff) + 0xc8) as u8, (v & 0xff) as u8],
            -262_144..=262_143 => vec![
                (((v >> 16) & 0xff) + 0xd4) as u8,
                ((v >> 8) & 0xff) as u8,
                (v & 0xff) as u8,
            ],
            _ => vec![
                b'I',
                (v >> 24 & 0xff) as u8,
                (v >> 16 & 0xff) as u8,
                (v >> 8 & 0xff) as u8,
                (v & 0xff) as u8,
            ],
        };

        self.writer.write_all(&bytes)?;
        Ok(())
    }

    fn serialize_binary(&mut self, v: &[u8]) -> Result<()> {
        if v.len() < 16 {
            self.writer
                .write(&[(v.len() - 0x20) as u8])
                .and_then(|_| self.writer.write_all(&v))?;
        } else {
            for (last, chunk) in v.chunks(0xffff).identify_last() {
                let flag = if last { b'B' } else { b'A' };
                let len_bytes = (v.len() as u16).to_be_bytes();
                self.writer.write_all(&[flag]).and_then(|_| {
                    self.writer
                        .write_all(&len_bytes)
                        .and_then(|_| self.writer.write_all(chunk))
                })?;
            }
        }
        Ok(())
    }

    // Serialize String to bytes, format as
    //    string ::= x52 b1 b0 <utf8-data> string
    //    ::= S b1 b0 <utf8-data>
    //    ::= [x00-x1f] <utf8-data>
    //    ::= [x30-x33] b0 <utf8-data>
    fn serialize_string(&mut self, v: &str) -> Result<()> {
        let will_write_bytes = v.as_bytes();
        let will_write_bytes_len = will_write_bytes.len();
        if will_write_bytes_len <= 0x1f {
            self.writer
                .write_all(&[will_write_bytes_len as u8])
                .and_then(|_| self.writer.write_all(will_write_bytes))?;
        } else if will_write_bytes_len < 1024 {
            self.writer
                .write_all(&[
                    (0x30 + ((will_write_bytes_len >> 8) & 0xff)) as u8,
                    (will_write_bytes_len & 0xff) as u8,
                ])
                .and_then(|_| self.writer.write_all(will_write_bytes))?;
        } else {
            // Split long string to many chunks
            for (last, chunk) in will_write_bytes.chunks(0xffff).identify_last() {
                let flag = if last { b'S' } else { 0x52 };
                let len_bytes = (v.len() as u16).to_be_bytes();
                let res = self.writer.write_all(&[flag]).and_then(|_| {
                    self.writer
                        .write_all(&len_bytes)
                        .and_then(|_| self.writer.write_all(chunk))
                });
                if let Err(e) = res {
                    return Err(Error::IoError(e));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Serializer;
    use crate::value::Value;
    use crate::value::Value::Int;

    fn test_encode_ok(value: Value, target: &[u8]) {
        let mut ser = Serializer::new(Vec::new());
        assert!(ser.serialize_value(&value).is_ok());
        assert_eq!(ser.writer.to_vec(), target);
    }

    #[test]
    fn test_encode_int() {
        test_encode_ok(Int(0), &[0x90 as u8]);
        test_encode_ok(Int(-16), &[0x80]);
        test_encode_ok(Int(47), &[0xbf]);
        test_encode_ok(Int(48), &[0xc8, 0x30]);

        test_encode_ok(Int(-2048), &[0xc0, 0x00]);
        test_encode_ok(Int(-256), &[0xc7, 0x00]);
        test_encode_ok(Int(2047), &[0xcf, 0xff]);

        test_encode_ok(Int(-262144), &[0xd0, 0x00, 0x00]);
        test_encode_ok(Int(262143), &[0xd7, 0xff, 0xff]);

        test_encode_ok(Int(262144), &[b'I', 0x00, 0x04, 0x00, 0x00])
    }

    #[test]
    fn test_encde_string() {
        // TODO(lynskylate): Add more test for encode string
        test_encode_ok(Value::String(String::new()), &[0x00]);
        test_encode_ok(
            Value::String(vec!['a'; 0x1f].into_iter().collect()),
            &[&[0x1f as u8], vec!['a' as u8; 0x1f].as_slice()].concat(),
        );
    }

    #[test]
    fn test_encode_bool() {
        test_encode_ok(Value::Bool(true), &[b'T']);
        test_encode_ok(Value::Bool(false), &[b'F']);
    }

    #[test]
    fn test_encode_null() {
        test_encode_ok(Value::Null, &[b'N']);
    }

    #[test]
    fn test_encode_long() {
        test_encode_ok(Value::Long(262144), &[0x59, 0x00, 0x04, 0x00, 0x00]);
        test_encode_ok(
            Value::Long(i32::max_value() as i64),
            &[0x59, 0x7f, 0xff, 0xff, 0xff],
        );
        test_encode_ok(
            Value::Long(i32::max_value() as i64 + 1),
            &[b'L', 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00],
        );
    }
}
