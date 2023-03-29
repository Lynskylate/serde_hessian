use std::fmt;

use hessian_rs::{de::Deserializer as HessianDecoder, ByteCodecType};

use crate::error::Error;
use hessian_rs::constant::List as ListType;
use hessian_rs::Value;
use serde::de::{self, Visitor};

pub struct Deserializer<R: AsRef<[u8]>> {
    de: HessianDecoder<R>,
}

struct SeqAccess<'a, R: AsRef<[u8]>> {
    de: &'a mut Deserializer<R>,
    name: Option<String>,
    len: Option<usize>,
    inx: usize,
}

impl<'a, R: AsRef<[u8]>> fmt::Display for SeqAccess<'a, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "SeqAccess(class: {})",
                self.name.clone().unwrap_or("None".into())
            )
        )
    }
}

impl<'a, R: AsRef<[u8]>> SeqAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>, name: Option<String>, len: Option<usize>) -> Self {
        SeqAccess {
            de,
            name,
            len,
            inx: 0,
        }
    }
}

impl<'de, 'a, R: AsRef<[u8]>> de::SeqAccess<'de> for SeqAccess<'a, R> {
    type Error = Error;

    #[inline]
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.len.is_none() && self.de.de.peek_byte()? == b'Z' {
            return Ok(None);
        } else if self.len.is_some() && self.len.unwrap() == self.inx {
            return Ok(None);
        }
        let value = seed.deserialize(&mut *self.de)?;
        self.inx += 1;
        Ok(Some(value))
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        self.len
    }
}

impl<'de, R: AsRef<[u8]>> Deserializer<R> {
    pub fn new(de: HessianDecoder<R>) -> Self {
        Deserializer { de }
    }

    pub fn from_bytes(s: R) -> Result<Self, Error> {
        Ok(Deserializer::new(HessianDecoder::new(s)))
    }
}

impl<'de, 'a, R> serde::Deserializer<'de> for &'a mut Deserializer<R>
where
    R: AsRef<[u8]>,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.de.peek_byte_code_type()? {
            hessian_rs::ByteCodecType::True => self.deserialize_bool(visitor),
            hessian_rs::ByteCodecType::False => self.deserialize_bool(visitor),
            hessian_rs::ByteCodecType::Null => self.deserialize_unit(visitor),
            hessian_rs::ByteCodecType::Int(_) => self.deserialize_i32(visitor),
            hessian_rs::ByteCodecType::Long(_) => self.deserialize_i64(visitor),
            hessian_rs::ByteCodecType::Double(_) => self.deserialize_f64(visitor),
            hessian_rs::ByteCodecType::Binary(_) => self.deserialize_bytes(visitor),
            hessian_rs::ByteCodecType::String(_) => self.deserialize_string(visitor),
            hessian_rs::ByteCodecType::List(_) => self.deserialize_seq(visitor),
            hessian_rs::ByteCodecType::Map(_) => self.deserialize_map(visitor),
            hessian_rs::ByteCodecType::Definition => {
                self.de.read_definition()?;
                self.deserialize_any(visitor)
            }
            hessian_rs::ByteCodecType::Date(_) => todo!(),
            hessian_rs::ByteCodecType::Object(_) => todo!(),
            hessian_rs::ByteCodecType::Ref => Err(Error::UnSupportedRefType),
            hessian_rs::ByteCodecType::Unknown => todo!(),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.de.read_value()? {
            hessian_rs::Value::Bool(v) => visitor.visit_bool(v),
            _ => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                "deserialize bool expect a bool value".into(),
            ))),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_i32(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_i32(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.de.read_value()? {
            hessian_rs::Value::Int(v) => visitor.visit_i32(v),
            hessian_rs::Value::Long(v) => visitor.visit_i32(v as i32),
            hessian_rs::Value::Double(v) => visitor.visit_i32(v as i32),
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                format!(
                    "deserialize i32 expect a i32 value, but get {}",
                    v.to_string()
                )
                .into(),
            ))),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.de.read_value()? {
            hessian_rs::Value::Int(v) => visitor.visit_i64(v as i64),
            hessian_rs::Value::Long(v) => visitor.visit_i64(v),
            hessian_rs::Value::Double(v) => visitor.visit_i64(v as i64),
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                format!(
                    "deserialize i64 expect a i64 value, but get {}",
                    v.to_string()
                )
                .into(),
            ))),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.de.read_value()? {
            hessian_rs::Value::Int(v) => visitor.visit_u8(v as u8),
            hessian_rs::Value::Long(v) => visitor.visit_u8(v as u8),
            // Allow deserializing a double/bytes(length is 1) as a u8
            hessian_rs::Value::Double(v) => visitor.visit_u8(v as u8),
            hessian_rs::Value::Bytes(b) => {
                if b.len() != 1 {
                    Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                        format!(
                            "deserialize u8 expect a u8 value, but get bytes, size is {}",
                            b.len()
                        )
                        .into(),
                    )))
                } else {
                    visitor.visit_char(b[0] as char)
                }
            }
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                format!(
                    "deserialize u8 expect a int/long value, but get {}",
                    v.to_string()
                )
                .into(),
            ))),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.de.read_value()? {
            hessian_rs::Value::Int(v) => visitor.visit_u16(v as u16),
            hessian_rs::Value::Long(v) => visitor.visit_u16(v as u16),
            hessian_rs::Value::Double(v) => visitor.visit_u16(v as u16),
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                format!(
                    "deserialize u16 expect a int/long value, but get {}",
                    v.to_string()
                )
                .into(),
            ))),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.de.read_value()? {
            hessian_rs::Value::Int(v) => visitor.visit_u32(v as u32),
            hessian_rs::Value::Long(v) => visitor.visit_u32(v as u32),
            hessian_rs::Value::Double(v) => visitor.visit_u32(v as u32),
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                format!(
                    "deserialize u32 expect a int/long value, but get {}",
                    v.to_string()
                )
                .into(),
            ))),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.de.read_value()? {
            hessian_rs::Value::Int(v) => visitor.visit_u64(v as u64),
            hessian_rs::Value::Long(v) => visitor.visit_u64(v as u64),
            hessian_rs::Value::Double(v) => visitor.visit_u64(v as u64),
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                format!(
                    "deserialize u64 expect a int/long value, but get {}",
                    v.to_string()
                )
                .into(),
            ))),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.de.read_value()? {
            hessian_rs::Value::Int(v) => visitor.visit_f32(v as f32),
            hessian_rs::Value::Long(v) => visitor.visit_f32(v as f32),
            hessian_rs::Value::Double(v) => visitor.visit_f32(v as f32),
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                format!(
                    "deserialize f32 expect a int/long value, but get {}",
                    v.to_string()
                )
                .into(),
            ))),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.de.read_value()? {
            hessian_rs::Value::Int(v) => visitor.visit_f64(v as f64),
            hessian_rs::Value::Long(v) => visitor.visit_f64(v as f64),
            hessian_rs::Value::Double(v) => visitor.visit_f64(v),
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                format!(
                    "deserialize f64 expect a int/long value, but get {}",
                    v.to_string()
                )
                .into(),
            ))),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.de.read_value()? {
            hessian_rs::Value::Bytes(b) => {
                if b.len() != 1 {
                    Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                        format!(
                            "deserialize char expect a char value, but get bytes, size is {}",
                            b.len()
                        )
                        .into(),
                    )))
                } else {
                    visitor.visit_char(b[0] as char)
                }
            }
            hessian_rs::Value::Long(v) => {
                if v < 256 {
                    visitor.visit_char(v as u8 as char)
                } else {
                    Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                        format!(
                            "deserialize char expect a char value, but get {}",
                            v.to_string()
                        )
                        .into(),
                    )))
                }
            }
            hessian_rs::Value::Int(v) => {
                if v < 256 {
                    visitor.visit_char(v as u8 as char)
                } else {
                    Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                        format!(
                            "deserialize char expect a char value, but get {}",
                            v.to_string()
                        )
                        .into(),
                    )))
                }
            }
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                format!(
                    "deserialize char expect a char value, but get {}",
                    v.to_string()
                )
                .into(),
            ))),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.de.read_value()? {
            hessian_rs::Value::Bytes(b) => {
                let s = String::from_utf8(b)?;
                visitor.visit_str(&s)
            }
            hessian_rs::Value::String(s) => visitor.visit_str(&s),
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                format!(
                    "deserialize str expect a string value, but get {}",
                    v.to_string()
                )
                .into(),
            ))),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.de.read_value()? {
            hessian_rs::Value::Bytes(b) => {
                let s = String::from_utf8(b)?;
                visitor.visit_string(s)
            }
            hessian_rs::Value::String(s) => visitor.visit_string(s),
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                format!(
                    "deserialize string expect a string value, but get {}",
                    v.to_string()
                )
                .into(),
            ))),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.de.read_value()? {
            hessian_rs::Value::Bytes(b) => visitor.visit_bytes(&b),
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                format!(
                    "deserialize bytes expect a bytes value, but get {}",
                    v.to_string()
                )
                .into(),
            ))),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.de.read_value()? {
            hessian_rs::Value::Bytes(b) => visitor.visit_byte_buf(b),
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                format!(
                    "deserialize byte_buf expect a bytes value, but get {}",
                    v.to_string()
                )
                .into(),
            ))),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.de.peek_byte_code_type()? {
            ByteCodecType::Null => {
                self.de.read_value()?.as_null();
                visitor.visit_unit()
            }
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.de.peek_byte_code_type()? {
            ByteCodecType::Null => {
                self.de.read_value()?.as_null();
                visitor.visit_unit()
            }
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                format!(
                    "deserialize unit expect a null tag, but get tag {}",
                    v.to_string()
                )
                .into(),
            ))),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let tag = self.de.read_byte()?;
        match ByteCodecType::from(tag) {
            ByteCodecType::List(ListType::FixedLength(typed)) => {
                let type_name = if typed {
                    Some(self.de.read_type()?)
                } else {
                    None
                };
                let length = match self.de.read_value()? {
                    Value::Int(l) => l as usize,
                    v => {
                        return Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                            format!(
                                "deserialize seq length expect a int value, but get {}",
                                v.to_string()
                            )
                            .into(),
                        )))
                    }
                };
                visitor.visit_seq(SeqAccess::new(self, type_name, Some(length)))
            }
            ByteCodecType::List(ListType::ShortFixedLength(typed, length)) => {
                let type_name = if typed {
                    Some(self.de.read_type()?)
                } else {
                    None
                };
                visitor.visit_seq(SeqAccess::new(self, type_name, Some(length)))
            }
            ByteCodecType::List(ListType::VarLength(typed)) => {
                let type_name = if typed {
                    Some(self.de.read_type()?)
                } else {
                    None
                };
                visitor.visit_seq(SeqAccess::new(self, type_name, None))
            }
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                format!(
                    "deserialize seq expect a list or map tag, but get tag {}",
                    v.to_string()
                )
                .into(),
            ))),
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }
}

// fn from_trait<'de, R, T>(read: R) -> Result<T, Error>
// where
//     R: AsRef<[u8]>,
//     T: de::Deserialize<'de>,
// {
//     let mut de = Deserializer::from_bytes(read);
//     let value = de::Deserialize::deserialize(&mut de)?;

//     // Make sure the whole stream has been consumed.
//     de.end()?;
//     Ok(value)
// }

#[cfg(test)]
mod tests {
    use crate::de::Deserializer;
    use serde::Deserialize;

    fn test_decode_ok<'a, T>(rdr: &[u8], target: T)
    where
        T: Deserialize<'a> + std::cmp::PartialEq + std::fmt::Debug,
    {
        let mut de = Deserializer::from_bytes(rdr).unwrap();
        let t = T::deserialize(&mut de).unwrap();
        assert_eq!(t, target);
    }
    #[test]
    fn test_basic_type() {
        // BasicType I32
        {
            test_decode_ok(&[b'I', 0x00, 0x00, 0x00, 0x00], 0);
            test_decode_ok(&[0x90u8], 0);
            test_decode_ok(&[0x80u8], -16);
            test_decode_ok(&[0xbfu8], 47);
        }

        // BasicType i64
        {
            test_decode_ok(&[0x59, 0x80, 0x00, 0x00, 0x00], -2147483648 as i64);
            test_decode_ok(&[0x59, 0x7f, 0xff, 0xff, 0xff], 2147483647 as i64);

            test_decode_ok(&[0x59, 0x80, 0x00, 0x00, 0x00], -2147483648 as i32);
            test_decode_ok(&[0x59, 0x7f, 0xff, 0xff, 0xff], 2147483647 as i32);
        }

        // BasicType f32/f64
        {
            test_decode_ok(&[0x5b], 0 as i32);
            test_decode_ok(&[0x5b], 0.0);
            test_decode_ok(&[0x5c], 1.0);
            test_decode_ok(&[0x5d, 0x80], -128.0);
            test_decode_ok(&[0x5e, 0x00, 0x80], 128.0);
            test_decode_ok(&[0x5f, 0x00, 0x00, 0x2f, 0xda], 12.25);
            test_decode_ok(
                &[b'D', 0x40, 0x28, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00],
                12.25,
            );
        }

        {
            test_decode_ok(
                &[b'V', 0x04, b'[', b'i', b'n', b't', 0x92, 0x90, 0x91],
                vec![0, 1],
            );
            //Untyped variable list
            test_decode_ok(&[0x57, 0x90, 0x91, b'Z'], vec![0, 1]);
        }
    }
    #[test]
    fn test_newtype_struct() {
        #[derive(Deserialize, Debug)]
        struct Test(i32);

        {
            let v = &[b'I', 0x00, 0x00, 0x00, 0x01];
            let mut de = Deserializer::from_bytes(v).unwrap();
            let t = Test::deserialize(&mut de).unwrap();
            assert_eq!(t.0, 1);
        }
    }
}
