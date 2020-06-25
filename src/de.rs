use std::collections::HashMap;
use std::io::{self, Cursor, Read, Seek, SeekFrom};

use byteorder::{BigEndian, ReadBytesExt};

use super::constant::{Binary, ByteCodecType, Date, Integer, List, Long};
use super::error::Error::SyntaxError;
use super::error::{ErrorKind, Result};
use super::value::Value;

pub struct Deserializer<R: AsRef<[u8]>> {
    buffer: Cursor<R>,
    type_references: Vec<String>,
    class_references: Vec<Defintion>,
}

#[derive(Debug, Clone)]
struct Defintion {
    name: String,
    fields: Vec<String>,
}


impl<R: AsRef<[u8]>> Deserializer<R> {
    pub fn new(rd: R) -> Deserializer<R> {
        Deserializer {
            buffer: Cursor::new(rd),
            type_references: Vec::new(),
            class_references: Vec::new(),
        }
    }

    fn error<T>(&self, err: ErrorKind) -> Result<T> {
        Err(SyntaxError(err))
    }

    #[inline]
    fn read_byte(&mut self) -> Result<u8> {
        Ok(self.buffer.read_u8()?)
    }

    #[inline]
    fn read_bytes(&mut self, n: usize) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        match self.buffer.by_ref().take(n as u64).read_to_end(&mut buf)? {
            m if m == n => Ok(buf),
            _ => Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Unexpected EOF").into()),
        }
    }

    #[inline]
    fn peek_byte(&mut self) -> Result<u8> {
        let tag = self.buffer.read_u8()?;
        self.buffer.seek(SeekFrom::Current(-1))?;
        Ok(tag)
    }

    fn read_definition(&mut self) -> Result<()> {
        // TODO(lynskylate@gmail.com): optimize error
        let name = match self.read_value() {
            Ok(Value::String(n)) => Ok(n),
            _ => self.error(ErrorKind::UnknownType),
        }?;
        let length = match self.read_value() {
            Ok(Value::Int(l)) => Ok(l),
            _ => self.error(ErrorKind::UnknownType),
        }?;

        let mut fields = Vec::new();

        for _ in 0..length {
            match self.read_value() {
                Ok(Value::String(s)) => fields.push(s),
                Ok(v) => {
                    return self.error(ErrorKind::UnExpectError(format!(
                        "Expect get string, but get {}",
                        &v
                    )))
                }
                _ => {
                    return self.error(ErrorKind::UnknownType);
                }
            }
        }

        self.class_references.push(Defintion {
            name: name,
            fields: fields,
        });
        Ok(())
    }

    fn read_object(&mut self) -> Result<Value> {
        if let Value::Int(i) = self.read_value()? {
            // TODO(lynskylate@gmail.com): Avoid copy
            let definition = self.class_references.get(i as usize)
                .ok_or(SyntaxError(ErrorKind::OutofDefinitionRange(i as usize)))?.clone();
            
            let fields_size = definition.fields.len();
            let mut map = HashMap::new();
            for i in 0..fields_size {
                let k = definition.fields[i].clone();
                let v = self.read_value()?;
                map.insert(Value::String(k), v);
            }
            Ok(Value::Map(map))
        } else {
            self.error(ErrorKind::MisMatchType)
        }
    }

    fn read_long_binary(&mut self, tag: u8) -> Result<Value> {
        let mut buf = Vec::new();
        let mut tag = tag;
        // Get non-final chunk starts with 'A'
        while tag == 0x41 {
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

    fn read_long(&mut self, l: Long) -> Result<Value> {
        match l {
            Long::DirectLong(b) => Ok(Value::Long(b as i64 - 0xe0)),
            Long::ByteLong(b) => {
                let b2 = self.read_byte()?;
                Ok(Value::Long(
                    i16::from_be_bytes([b.overflowing_sub(0xf8).0, b2]) as i64,
                ))
            }
            Long::ShortLong(b) => {
                let bs = self.read_bytes(2)?;
                Ok(Value::Long(
                    (i32::from_be_bytes([b.overflowing_sub(0x3c).0, bs[0], bs[1], 0x00]) >> 8)
                        as i64,
                ))
            }
            Long::Int32Long => Ok(Value::Long(self.buffer.read_i32::<BigEndian>()? as i64)),
            Long::NormalLong => Ok(Value::Long(self.buffer.read_i64::<BigEndian>()?)),
        }
    }

    fn read_double(&mut self, tag: u8) -> Result<Value> {
        let val = match tag {
            b'D' => self.buffer.read_f64::<BigEndian>()?,
            0x5b => 0.0,
            0x5c => 1.0,
            0x5d => self.buffer.read_i8()? as f64,
            0x5e => self.buffer.read_i16::<BigEndian>()? as f64,
            0x5f => (self.buffer.read_i32::<BigEndian>()? as f64) * 0.001,
            _ => todo!(),
        };
        Ok(Value::Double(val))
    }

    fn read_date(&mut self, d: Date) -> Result<Value> {
        let val = match d {
            Date::Millisecond => self.buffer.read_i64::<BigEndian>()?,
            Date::Minute => self.buffer.read_i32::<BigEndian>()? as i64 * 60000,
        };
        Ok(Value::Date(val))
    }

    fn read_utf8_string(&mut self, len: usize) -> Result<Vec<u8>> {
        let mut s = Vec::new();
        let mut len = len;
        while len > 0 {
            let byte = self.read_byte()?;
            match byte {
                0x00..=0x7f => s.push(byte),
                0xc2..=0xdf => {
                    s.push(byte);
                    s.push(self.read_byte()?);
                }
                0xe0..=0xef => {
                    s.push(byte);
                    let mut buf = [0; 2];
                    self.buffer.read_exact(&mut buf)?;
                    s.extend_from_slice(&buf);
                }
                0xf0..=0xf4 => {
                    s.push(byte);
                    let mut buf = [0; 3];
                    self.buffer.read_exact(&mut buf)?;
                    s.extend_from_slice(&buf);
                }
                _ => {}
            }
            len = len - 1;
        }
        Ok(s)
    }

    fn read_string_internal(&mut self, tag: u8) -> Result<Vec<u8>> {
        // TODO: remove unnecessary copying
        let mut buf = Vec::new();
        match tag {
            // ::= [x00-x1f] <utf8-data>         # string of length 0-31
            0x00..=0x1f => {
                let len = tag as usize - 0x00;
                buf.extend_from_slice(&self.read_utf8_string(len)?);
            }
            // ::= [x30-x34] <utf8-data>         # string of length 0-1023
            0x30..=0x33 => {
                let len = (tag as usize - 0x30) * 256 + self.read_byte()? as usize;
                buf.extend_from_slice(&self.read_utf8_string(len)?);
            }
            // x52 ('R') represents any non-final chunk
            0x52 => {
                let len = self.buffer.read_u16::<BigEndian>()? as usize;
                buf.extend_from_slice(&self.read_utf8_string(len)?);
                let next_tag = self.read_byte()?;
                buf.extend_from_slice(&self.read_string_internal(next_tag)?);
            }
            // x53 ('S') represents the final chunk
            0x53 => {
                let len = self.buffer.read_u16::<BigEndian>()? as usize;
                buf.extend_from_slice(&self.read_utf8_string(len)?);
            }
            _ => { /* should not happen */ }
        }
        Ok(buf)
    }

    fn read_string(&mut self, tag: u8) -> Result<Value> {
        let buf = self.read_string_internal(tag)?;
        let s = unsafe { String::from_utf8_unchecked(buf) };
        Ok(Value::String(s))
    }

    fn read_type(&mut self) -> Result<String> {
        match self.read_value() {
            Ok(Value::String(s)) => {
                self.type_references.push(s.clone());
                Ok(s)
            }
            Ok(Value::Int(i)) => {
                if let Some(res) = self.type_references.iter().nth(i as usize) {
                    Ok(res.clone())
                } else {
                    self.error(ErrorKind::OutofTypeRefRange(i as usize))
                }
            }
            Ok(_) => self.error(ErrorKind::MisMatchType),
            Err(e) => Err(e),
        }
    }

    fn read_varlength_map_interal(&mut self) -> Result<HashMap<Value, Value>> {
        let mut map = HashMap::new();
        let mut tag = self.peek_byte()?;
        while tag != b'Z' {
            let key = self.read_value()?;
            let val = self.read_value()?;
            map.insert(key, val);
            tag = self.peek_byte()?;
        }
        self.read_byte()?;
        Ok(map)
    }

    fn read_varlength_list_internal(&mut self) -> Result<Vec<Value>> {
        let mut tag = self.peek_byte()?;
        let mut list = Vec::new();
        while tag != b'Z' {
            list.push(self.read_value()?);
            tag = self.peek_byte()?;
        }
        self.read_byte()?;
        Ok(list)
    }

    fn read_exact_length_list_internal(&mut self, length: usize) -> Result<Vec<Value>> {
        let mut list = Vec::new();
        for _ in 0..length {
            list.push(self.read_value()?)
        }
        Ok(list)
    }

    fn read_list(&mut self, list: List) -> Result<Value> {
        // TODO(lynskylate@gmail.com): Should add list to reference, but i don't know any good way to deal with it
        match list {
            List::ShortFixedLengthList(typed, length) => {
                if typed {
                    self.read_type()?;
                }
                Ok(Value::List(self.read_exact_length_list_internal(length)?))
            }
            List::VarLengthList(typed) => {
                if typed {
                    self.read_type()?;
                }
                Ok(Value::List(self.read_varlength_list_internal()?))
            }
            List::FixedLengthList(typed) => {
                if typed {
                    self.read_type()?;
                }
                if let Value::Int(length) = self.read_value()? {
                    Ok(Value::List(
                        self.read_exact_length_list_internal(length as usize)?,
                    ))
                } else {
                    self.error(ErrorKind::MisMatchType)
                }
            }
        }
    }

    fn read_map(&mut self, typed: bool) -> Result<Value> {
        if typed {
            self.read_type()?;
        }
        Ok(Value::Map(self.read_varlength_map_interal()?))
    }

    fn read_ref(&mut self) -> Result<Value> {
        println!("ref ref");
        if let Value::Int(i) = self.read_value()? {
            Ok(Value::Ref(i as u32))
        } else {
            self.error(ErrorKind::MisMatchType)
        }
    }

    pub fn read_value(&mut self) -> Result<Value> {
        let v = self.read_byte()?;
        match ByteCodecType::from(v) {
            ByteCodecType::Int(i) => self.read_int(i),
            ByteCodecType::Long(l) => self.read_long(l),
            ByteCodecType::Double(d) => self.read_double(d),
            ByteCodecType::Date(d) => self.read_date(d),
            ByteCodecType::Binary(bin) => self.read_binary(bin),
            ByteCodecType::String(s) => self.read_string(s),
            ByteCodecType::List(l) => self.read_list(l),
            ByteCodecType::Map(typed) => self.read_map(typed),
            ByteCodecType::True => Ok(Value::Bool(true)),
            ByteCodecType::False => Ok(Value::Bool(false)),
            ByteCodecType::Null => Ok(Value::Null),
            ByteCodecType::Definition => {
                self.read_definition()?;
                self.read_value()
            },
            ByteCodecType::Ref => self.read_ref(),
            ByteCodecType::Object => self.read_object(),
            _ => self.error(ErrorKind::UnknownType),
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
    use std::collections::HashMap;

    fn test_decode_ok(rdr: &[u8], target: Value) {
        let mut de = Deserializer::new(rdr);
        let value = de.read_value().unwrap();
        assert_eq!(value, target);
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
    fn test_decode_long() {
        // -8 ~ 15
        test_decode_ok(&[0xe0], Value::Long(0));
        test_decode_ok(&[0xd8], Value::Long(-8));
        test_decode_ok(&[0xd9], Value::Long(-7));
        test_decode_ok(&[0xef], Value::Long(15));
        test_decode_ok(&[0xee], Value::Long(14));
        // -2048 ~ 2047
        test_decode_ok(&[0xf7, 0xf7], Value::Long(-9));
        test_decode_ok(&[0xf8, 0x10], Value::Long(16));
        test_decode_ok(&[0xf0, 0x00], Value::Long(-2048));
        test_decode_ok(&[0xff, 0xff], Value::Long(2047));
        // -262144 ~ 262143
        test_decode_ok(&[0x3f, 0xff, 0xff], Value::Long(262143));
        test_decode_ok(&[0x38, 0x00, 0x00], Value::Long(-262144));
        test_decode_ok(&[0x3c, 0x08, 0x00], Value::Long(2048));
        // -2147483648 ~ 2147483647
        test_decode_ok(&[0x59, 0x80, 0x00, 0x00, 0x00], Value::Long(-2147483648));
        test_decode_ok(&[0x59, 0x7f, 0xff, 0xff, 0xff], Value::Long(2147483647));
        // L
        test_decode_ok(
            &[0x4c, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00],
            Value::Long(2147483648),
        );
    }

    #[test]
    fn test_decode_double() {
        test_decode_ok(&[0x5b], Value::Double(0.0));
        test_decode_ok(&[0x5c], Value::Double(1.0));
        test_decode_ok(&[0x5d, 0x80], Value::Double(-128.0));
        test_decode_ok(&[0x5e, 0x00, 0x80], Value::Double(128.0));
        test_decode_ok(&[0x5f, 0x00, 0x00, 0x2f, 0xda], Value::Double(12.25));
        test_decode_ok(
            &[b'D', 0x40, 0x28, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00],
            Value::Double(12.25),
        );
    }

    #[test]
    fn test_decode_date() {
        test_decode_ok(
            &[0x4a, 0x00, 0x00, 0x00, 0xd0, 0x4b, 0x92, 0x84, 0xb8],
            Value::Date(894621091000),
        );
        test_decode_ok(&[0x4b, 0x4b, 0x92, 0x0b, 0xa0], Value::Date(76071745920000));
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

    #[test]
    fn test_read_list() {
        //Fixed length typed list
        test_decode_ok(
            &[b'V', 0x04, b'[', b'i', b'n', b't', 0x92, 0x90, 0x91],
            Value::List(vec![Value::Int(0), Value::Int(1)]),
        );
        //Untyped variable list
        test_decode_ok(
            &[0x57, 0x90, 0x91, b'Z'],
            Value::List(vec![Value::Int(0), Value::Int(1)]),
        );
    }

    #[test]
    fn test_read_map() {
        let mut map = HashMap::new();
        map.insert(Value::Int(1), Value::String("fee".to_string()));
        map.insert(Value::Int(16), Value::String("fie".to_string()));
        map.insert(Value::Int(256), Value::String("foe".to_string()));
        test_decode_ok(
            &[
                b'M', 0x13, b'c', b'o', b'm', b'.', b'c', b'a', b'u', b'c', b'h', b'o', b'.', b't',
                b'e', b's', b't', b'.', b'c', b'a', b'r', 0x91, 0x03, b'f', b'e', b'e', 0xa0, 0x03,
                b'f', b'i', b'e', 0xc9, 0x00, 0x03, b'f', b'o', b'e', b'Z',
            ],
            Value::Map(map.clone()),
        );

        test_decode_ok(
            &[
                b'H', 0x91, 0x03, b'f', b'e', b'e', 0xa0, 0x03, b'f', b'i', b'e', 0xc9, 0x00, 0x03,
                b'f', b'o', b'e', b'Z',
            ],
            Value::Map(map.clone()),
        );
    }

    #[test]
    fn test_read_object() {
        let mut map = HashMap::new();
        map.insert(Value::String("Color".to_string()), Value::String("red".to_string()));
        map.insert(Value::String("Model".to_string()), Value::String("corvette".to_string()));
        test_decode_ok(
            &[
                b'C', 0x0b, b'e', b'x', b'a', b'm', b'p', b'l', b'e', b'.', b'C', b'a', b'r',
                0x92, 0x05, b'C', b'o', b'l', b'o', b'r', 0x05, b'M', b'o', b'd', b'e', b'l',
                b'O', 0x90, 0x03, b'r', b'e', b'd', 0x08, b'c', b'o', b'r', b'v', b'e', b't', b't', b'e',
            ],
            Value::Map(map),
        );

    }

    #[test]
    fn test_read_ref() {
        let mut map = HashMap::new();
        map.insert(Value::String("head".to_string()), Value::Int(1));
        map.insert(Value::String("tail".to_string()), Value::Ref(0));
        test_decode_ok(&[
            b'C', 0x0a, b'L', b'i', b'n', b'k', b'e', b'd', b'L', b'i', b's', b't',
            0x92, 0x04, b'h', b'e', b'a', b'd', 0x04, b't', b'a', b'i', b'l',
            b'O', 0x90, 0x91, 0x51, 0x90
        ], Value::Map(map));
    }
}
