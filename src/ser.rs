use std::io;

use byteorder::{BigEndian, WriteBytesExt};
use indexmap::IndexSet;

use super::error::{Error, ErrorKind, Result};
use super::value::Value;

pub struct Serializer<W> {
    writer: W,
    type_cache: IndexSet<String>,
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
        Serializer {
            writer: writer,
            type_cache: IndexSet::new(),
        }
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
            Value::Date(d) => self.serialize_date(d),
            Value::Double(d) => self.serialize_double(d),
            Value::List(ref l) => self.serialize_list(l),
            _ => Err(Error::SyntaxError(ErrorKind::UnknownType)),
        }
    }

    fn write_type(&mut self, tp: &str) -> Result<()> {
        if let Some(inx) = self.type_cache.get_index_of(tp) {
            self.serialize_int(inx as i32)?;
        } else {
            self.serialize_string(tp)?;
            self.type_cache.insert(String::from(tp));
        }
        Ok(())
    }

    fn write_list_begin(&mut self, length: usize, tp: Option<&str>) -> Result<()> {
        if length <= 7 {
            if let Some(tp) = tp {
                self.writer.write_u8((0x70 + length) as u8)?;
                self.write_type(tp)?;
            } else {
                self.writer.write_u8((0x78 + length) as u8)?;
            }
        } else {
            if let Some(tp) = tp {
                self.writer.write_u8(0x56)?;
                self.write_type(tp)?;
            } else {
                self.writer.write_u8(0x58)?;
            }
            self.serialize_int(length as i32)?;
        }

        Ok(())
    }

    fn serialize_list_with_type(&mut self, list: Vec<Value>, tp: &str) -> Result<()> {
        self.write_list_begin(list.len(), Some(tp))?;
        for i in list.iter() {
            self.serialize_value(i)?;
        }
        Ok(())
    }

    fn serialize_list(&mut self, list: &Vec<Value>) -> Result<()> {
        self.write_list_begin(list.len(), None)?;
        for i in list.iter() {
            self.serialize_value(i)?;
        }
        Ok(())
    }

    fn serialize_date(&mut self, d: i64) -> Result<()> {
        self.writer.write_all(&[0x4a])?;
        self.writer.write_i64::<BigEndian>(d)?;
        Ok(())
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
            -8..=15 => vec![(0xe0 + v) as u8],
            -2048..=2047 => vec![(((v >> 8) + 0xf8) & 0xff) as u8, (v & 0xff) as u8],
            -262_144..=262_143 => vec![
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

    fn serialize_double(&mut self, v: f64) -> Result<()> {
        let int_v = v.trunc() as i32;
        if int_v as f64 == v {
            match int_v {
                0 => self.writer.write_u8(0x5b)?,
                1 => self.writer.write_u8(0x5c)?,
                -128..=127 => {
                    self.writer.write_u8(0x5d)?;
                    self.writer.write_u8(int_v as u8)?;
                }
                -32768..=32767 => {
                    self.writer.write_u8(0x5e)?;
                    self.writer.write_i16::<BigEndian>(int_v as i16)?;
                }
                _ => {}
            }
        } else {
            let mills = v * 1000.0;
            if mills * 0.001 == v {
                self.writer.write_u8(0x5f)?;
                self.writer.write_i32::<BigEndian>(mills as i32)?;
            } else {
                self.writer.write_u8(0x44)?;
                self.writer.write_f64::<BigEndian>(v)?;
            }
        }
        Ok(())
    }

    fn serialize_binary(&mut self, v: &[u8]) -> Result<()> {
        if v.len() < 16 {
            self.writer.write(&[(v.len() - 0x20) as u8])?;
            self.writer.write_all(&v)?;
        } else {
            for (last, chunk) in v.chunks(0xffff).identify_last() {
                let flag = if last { b'B' } else { b'A' };
                let len_bytes = (v.len() as u16).to_be_bytes();
                self.writer.write_all(&[flag])?;
                self.writer.write_all(&len_bytes)?;
                self.writer.write_all(chunk)?
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
        const MAX_CHUNK_BYTE_SIZE: u32 = 0x8000;
        let bytes = v.as_bytes();
        let mut len = 0;
        let mut offset = 0;
        let mut last = 0;
        let mut i = 0;
        while i < bytes.len() {
            len += 1;
            let byte = bytes[i];
            if byte & 0x80 > 0 {
                // more than one byte for this char
                if byte & 0xe0 == 0xc0 {
                    i += 2;
                } else if byte & 0xf0 == 0xe0 {
                    i += 3;
                } else {
                    i += 4;
                }
            } else {
                i += 1;
            }
            if len >= MAX_CHUNK_BYTE_SIZE {
                self.writer.write_u8(b'R')?;
                self.writer.write_u16::<BigEndian>(len as u16)?;
                self.writer.write_all(&bytes[offset..i - last])?;
                len = 0;
                offset += i;
                last = i;
            }
        }
        match len {
            0..=31 => self.writer.write_u8(len as u8)?,
            32..=1023 => self
                .writer
                .write_all(&[(0x30 + ((len >> 8) & 0xff)) as u8, (len & 0xff) as u8])?,
            _ => {
                self.writer.write_u8(b'S')?;
                self.writer.write_u16::<BigEndian>(len as u16)?;
            }
        }
        self.writer.write_all(&bytes[offset..i - last])?;
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
        assert_eq!(ser.writer.to_vec(), target, "{:?} encode error", value);
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
        test_encode_ok(Value::Long(0), &[0xe0]);
        test_encode_ok(Value::Long(-8), &[0xd8]);
        test_encode_ok(Value::Long(-7), &[0xd9]);
        test_encode_ok(Value::Long(15), &[0xef]);
        test_encode_ok(Value::Long(-9), &[0xf7, 0xf7]);
        test_encode_ok(Value::Long(16), &[0xf8, 0x10]);
        test_encode_ok(Value::Long(255), &[0xf8, 0xff]);
        test_encode_ok(Value::Long(-2048), &[0xf0, 0x00]);
        test_encode_ok(Value::Long(262143), &[0x3f, 0xff, 0xff]);
        test_encode_ok(Value::Long(-262144), &[0x38, 0x00, 0x00]);
        test_encode_ok(Value::Long(2048), &[0x3c, 0x08, 0x00]);
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

    #[test]
    fn test_encode_double() {
        test_encode_ok(Value::Double(0.0), &[0x5b]);
        test_encode_ok(Value::Double(1.0), &[0x5c]);
        test_encode_ok(Value::Double(127.0), &[0x5d, 0x7f]);
        test_encode_ok(Value::Double(-32768.0), &[0x5e, 0x80, 0x00]);
        test_encode_ok(Value::Double(12.25), &[0x5f, 0x00, 0x00, 0x2f, 0xda]);
        test_encode_ok(
            Value::Double(32767.99999),
            &[0x44, 0x40, 0xdf, 0xff, 0xff, 0xff, 0xd6, 0x0e, 0x95],
        );
    }

    #[test]
    fn test_encode_date() {
        test_encode_ok(
            Value::Date(894621091000),
            &[0x4a, 0x00, 0x00, 0x00, 0xd0, 0x4b, 0x92, 0x84, 0xb8],
        )
    }

    #[test]
    fn test_encode_type() {
        let mut ser = Serializer::new(Vec::new());
        ser.serialize_list_with_type(vec![Value::Int(1)], "test.list")
            .unwrap();
        assert_eq!(ser.type_cache.len(), 1);
        assert_eq!(ser.type_cache.get_index_of("test.list"), Some(0));
        ser.serialize_list_with_type(vec![Value::Int(2)], "test.list")
            .unwrap();
        assert_eq!(ser.type_cache.len(), 1);
    }
}
