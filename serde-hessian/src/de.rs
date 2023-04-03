use std::fmt;

use hessian_rs::{de::Deserializer as HessianDecoder, ByteCodecType};

use crate::error::Error;
use hessian_rs::constant::List as ListType;
use hessian_rs::Value;
use serde::de::{self, IntoDeserializer, Visitor};

pub struct Deserializer<R: AsRef<[u8]>> {
    de: HessianDecoder<R>,
}

struct MapAccess<'a, R: AsRef<[u8]>> {
    de: &'a mut Deserializer<R>,
    name: Option<String>,
}

struct SeqAccess<'a, R: AsRef<[u8]>> {
    de: &'a mut Deserializer<R>,
    name: Option<String>,
    len: Option<usize>,
    inx: usize,
}

struct EnumAccess<'a, R: AsRef<[u8]>> {
    de: &'a mut Deserializer<R>,
}

impl<'a, R: AsRef<[u8]>> EnumAccess<'a, R> {
    pub fn new(de: &'a mut Deserializer<R>) -> Self {
        EnumAccess { de }
    }
}

impl<'de, 'a, R: AsRef<[u8]>> de::EnumAccess<'de> for EnumAccess<'a, R> {
    type Error = Error;

    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

impl<'de, 'a, R: AsRef<[u8]>> de::VariantAccess<'de> for EnumAccess<'a, R> {
    type Error = Error;

    // If the `Visitor` expected this variant to be a unit variant, the input
    // should have been the plain string case handled in `deserialize_enum`.
    fn unit_variant(self) -> Result<(), Self::Error> {
        unreachable!("unit_variant")
    }

    // Newtype variants are represented in JSON as `{ NAME: VALUE }` so
    // deserialize the value here.
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }` so
    // deserialize the sequence of data here.
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }` so
    // deserialize the inner map here.
    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self.de, visitor)
    }
}

impl<'a, R: AsRef<[u8]>> MapAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>, name: Option<String>) -> Self {
        MapAccess { de, name }
    }
}

impl<'de, 'a, R: AsRef<[u8]>> de::MapAccess<'de> for MapAccess<'a, R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.de.de.peek_byte()? == b'Z' {
            self.de.de.read_byte()?;
            Ok(None)
        } else {
            Ok(Some(seed.deserialize(&mut *self.de)?))
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let v = seed.deserialize(&mut *self.de)?;
        Ok(v)
    }
}

impl<'a, R: AsRef<[u8]>> fmt::Display for MapAccess<'a, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MapAccess(class: {})",
            self.name.clone().unwrap_or("None".into())
        )
    }
}

impl<'a, R: AsRef<[u8]>> fmt::Display for SeqAccess<'a, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SeqAccess(class: {})",
            self.name.clone().unwrap_or("None".into())
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
        let end = if self.len.is_some() {
            self.len.unwrap() == self.inx
        } else {
            self.de.de.peek_byte()? == b'Z'
        };

        if end {
            if self.len.is_none() {
                // read 'Z'
                self.de.de.read_byte()?;
            }

            return Ok(None);
        }
        let value = seed.deserialize(&mut *self.de)?;
        self.inx += 1;
        Ok(Some(value))
    }

    #[inline(always)]
    fn size_hint(&self) -> Option<usize> {
        if self.len.is_some() {
            Some(self.len.unwrap() - self.inx)
        } else {
            None
        }
    }
}

impl<R: AsRef<[u8]>> Deserializer<R> {
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
                self.de.read_byte()?;
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
                format!("deserialize i32 expect a i32 value, but get {}", v),
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
                format!("deserialize i64 expect a i64 value, but get {}", v),
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
                        ),
                    )))
                } else {
                    visitor.visit_char(b[0] as char)
                }
            }
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                format!("deserialize u8 expect a int/long value, but get {}", v),
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
                format!("deserialize u16 expect a int/long value, but get {}", v),
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
                format!("deserialize u32 expect a int/long value, but get {}", v),
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
                format!("deserialize u64 expect a int/long value, but get {}", v),
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
                format!("deserialize f32 expect a int/long value, but get {}", v),
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
                format!("deserialize f64 expect a int/long value, but get {}", v),
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
                        ),
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
                        format!("deserialize char expect a char value, but get {}", v),
                    )))
                }
            }
            hessian_rs::Value::Int(v) => {
                if v < 256 {
                    visitor.visit_char(v as u8 as char)
                } else {
                    Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                        format!("deserialize char expect a char value, but get {}", v),
                    )))
                }
            }
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                format!("deserialize char expect a char value, but get {}", v),
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
                format!("deserialize str expect a string value, but get {}", v),
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
                format!("deserialize string expect a string value, but get {}", v),
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
                format!("deserialize bytes expect a bytes value, but get {}", v),
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
                format!("deserialize byte_buf expect a bytes value, but get {}", v),
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
                format!("deserialize unit expect a null tag, but get tag {}", v),
            ))),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
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
                            format!("deserialize seq length expect a int value, but get {}", v),
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
                    v
                ),
            ))),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
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
        let tag = self.de.read_byte()?;
        match ByteCodecType::from(tag) {
            ByteCodecType::Map(typed) => {
                let type_name = if typed {
                    Some(self.de.read_type()?)
                } else {
                    None
                };
                visitor.visit_map(MapAccess::new(self, type_name))
            }
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                format!("deserialize map expect a map tag, but get tag {}", v),
            ))),
        }
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
        let tag = self.de.read_byte()?;
        match ByteCodecType::from(tag) {
            ByteCodecType::Map(typed) => {
                let type_name = if typed {
                    Some(self.de.read_type()?)
                } else {
                    None
                };
                visitor.visit_map(MapAccess::new(self, type_name))
            }
            ByteCodecType::Definition => {
                self.de.read_definition()?;
                self.deserialize_struct(name, fields, visitor)
            }
            ByteCodecType::Object(o) => {
                // todo: check object type and fields
                let def_len = self.de.read_definition_id(o)?.fields.len();
                visitor.visit_seq(SeqAccess::new(self, None, Some(def_len)))
            }
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                format!("deserialize map expect a map tag, but get tag {}", v),
            ))),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let tag = self.de.peek_byte()?;
        match ByteCodecType::from(tag) {
            ByteCodecType::String(_) => {
                let value = self.de.read_value()?;
                visitor.visit_enum(value.as_str().unwrap().into_deserializer())
            }
            ByteCodecType::Map(typed) => {
                self.de.read_byte()?;
                if typed {
                    self.de.read_type()?;
                }
                visitor.visit_enum(EnumAccess::new(self))
            }
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                format!("deserialize enum can't support tag {}", v),
            ))),
        }
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
        self.deserialize_any(visitor)
    }
}

pub fn from_slice<'de, R, T>(read: R) -> Result<T, Error>
where
    R: AsRef<[u8]>,
    T: de::Deserialize<'de>,
{
    let mut de = Deserializer::from_bytes(read)?;
    let value = T::deserialize(&mut de)?;

    Ok(value)
}

#[cfg(test)]
mod tests {
    use crate::de::from_slice;
    use crate::de::Deserializer;
    use serde::Deserialize;
    use std::collections::HashMap;

    fn test_decode_ok<'a, T>(rdr: &[u8], target: T)
    where
        T: Deserialize<'a> + std::cmp::PartialEq + std::fmt::Debug,
    {
        let t: T = from_slice(rdr).unwrap();
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
            test_decode_ok(&[0x59, 0x80, 0x00, 0x00, 0x00], -2147483648_i64);
            test_decode_ok(&[0x59, 0x7f, 0xff, 0xff, 0xff], 2147483647_i64);

            test_decode_ok(&[0x59, 0x80, 0x00, 0x00, 0x00], -2147483648_i32);
            test_decode_ok(&[0x59, 0x7f, 0xff, 0xff, 0xff], 2147483647_i32);
        }

        // null
        {
            test_decode_ok(&[b'N'], ());
        }

        {
            test_decode_ok(&[b'N'], None::<()>);
        }

        // BasicType f32/f64
        {
            test_decode_ok(&[0x5b], 0_i32);
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
    fn test_basic_object_type() {
        {
            test_decode_ok(
                &[b'V', 0x04, b'[', b'i', b'n', b't', 0x92, 0x90, 0x91],
                vec![0, 1],
            );
            //Untyped variable list
            test_decode_ok(&[0x57, 0x90, 0x91, b'Z'], vec![0, 1]);
        }

        {
            test_decode_ok(&[b'T'], true);
            test_decode_ok(&[b'F'], false);
        }

        {
            let mut map = HashMap::new();
            map.insert(1, "fee".to_string());
            map.insert(16, "fie".to_string());
            map.insert(256, "foe".to_string());
            test_decode_ok(
                &[
                    b'M', 0x13, b'c', b'o', b'm', b'.', b'c', b'a', b'u', b'c', b'h', b'o', b'.',
                    b't', b'e', b's', b't', b'.', b'c', b'a', b'r', 0x91, 0x03, b'f', b'e', b'e',
                    0xa0, 0x03, b'f', b'i', b'e', 0xc9, 0x00, 0x03, b'f', b'o', b'e', b'Z',
                ],
                map.clone(),
            );

            test_decode_ok(
                &[
                    b'H', 0x91, 0x03, b'f', b'e', b'e', 0xa0, 0x03, b'f', b'i', b'e', 0xc9, 0x00,
                    0x03, b'f', b'o', b'e', b'Z',
                ],
                map.clone(),
            );
        }
    }

    #[test]
    fn test_basic_struct() {
        #[derive(Debug, PartialEq, Deserialize, Clone)]
        #[serde(rename = "example.Car", rename_all = "PascalCase")]
        struct Car {
            color: String,
            model: String,
        }

        let car = Car {
            color: "red".to_string(),
            model: "corvette".to_string(),
        };

        // deserialize struct from map
        test_decode_ok(
            &[
                b'H', 0x05, b'C', b'o', b'l', b'o', b'r', 0x03, b'r', b'e', b'd', 0x05, b'M', b'o',
                b'd', b'e', b'l', 0x08, b'c', b'o', b'r', b'v', b'e', b't', b't', b'e', b'Z',
            ],
            car.clone(),
        );

        // deserialize struct from object data
        test_decode_ok(
            &[
                b'C', 0x0b, b'e', b'x', b'a', b'm', b'p', b'l', b'e', b'.', b'C', b'a', b'r', 0x92,
                0x05, b'C', b'o', b'l', b'o', b'r', 0x05, b'M', b'o', b'd', b'e', b'l', b'O', 0x90,
                0x03, b'r', b'e', b'd', 0x08, b'c', b'o', b'r', b'v', b'e', b't', b't', b'e',
            ],
            car,
        );
    }

    #[test]
    fn test_enum() {
        #[derive(Deserialize, PartialEq, Debug)]
        enum E {
            Unit,
            Newtype(u32),
            Tuple(u32, u32),
            Struct { a: u32 },
        }

        test_decode_ok(&[0x04, b'U', b'n', b'i', b't'], E::Unit);
        test_decode_ok(
            &[
                b'H', 0x07, b'N', b'e', b'w', b't', b'y', b'p', b'e', 0x91, b'Z',
            ],
            E::Newtype(1),
        );
        test_decode_ok(
            &[
                b'H', 0x05, b'T', b'u', b'p', b'l', b'e', 0x57, 0x91, 0x91, b'Z',
            ],
            E::Tuple(1, 1),
        );
        test_decode_ok(
            &[
                b'H', 0x06, b'S', b't', b'r', b'u', b'c', b't', b'H', 0x01, b'a', 0x91, b'Z', b'Z',
            ],
            E::Struct { a: 1 },
        );
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
