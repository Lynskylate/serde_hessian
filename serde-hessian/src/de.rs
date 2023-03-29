use hessian_rs::de::Deserializer;

use crate::error::Error;
use serde::de;


pub struct Decoder<R: AsRef<[u8]>> {
    de: Deserializer<R>,
}

impl<'de, 'a, R> serde::Deserializer<'de> for &'a mut Decoder<R>
where R: AsRef<[u8]> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        match self.de.peek_byte_code_type()? {
            hessian_rs::ByteCodecType::True => self.deserialize_bool(visitor),
            hessian_rs::ByteCodecType::False => self.deserialize_bool(visitor),
            hessian_rs::ByteCodecType::Null => self.deserialize_unit(visitor),
            hessian_rs::ByteCodecType::Int(_) => self.deserialize_i32(visitor),
            hessian_rs::ByteCodecType::Long(_) => self.deserialize_i64(visitor),
            hessian_rs::ByteCodecType::Double(_) => self.deserialize_f64(visitor),
            hessian_rs::ByteCodecType::Binary(_) => self.deserialize_bytes(visitor),
            hessian_rs::ByteCodecType::String(_) => self.deserialize_string(visitor),
            hessian_rs::ByteCodecType::List(_) => todo!(),
            hessian_rs::ByteCodecType::Map(_) => todo!(),
            hessian_rs::ByteCodecType::Definition => todo!(),
            hessian_rs::ByteCodecType::Date(_) => todo!(),
            hessian_rs::ByteCodecType::Object(_) => todo!(),
            hessian_rs::ByteCodecType::Ref => todo!(),
            hessian_rs::ByteCodecType::Unknown => todo!(),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        match self.de.read_value()? {
            hessian_rs::Value::Bool(v) => visitor.visit_bool(v),
            _ => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType("deserialize bool expect a bool value".into()))),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        self.deserialize_i32(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        self.deserialize_i32(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        match self.de.read_value()? {
            hessian_rs::Value::Int(v) => visitor.visit_i32(v),
            hessian_rs::Value::Long(v) => visitor.visit_i32(v as i32),
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(format!("deserialize i32 expect a i32 value, but get {}", v.to_string()).into()))),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        match self.de.read_value()? {
            hessian_rs::Value::Int(v) => visitor.visit_i64(v as i64),
            hessian_rs::Value::Long(v) => visitor.visit_i64(v),
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(format!("deserialize i64 expect a i64 value, but get {}", v.to_string()).into()))),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        match self.de.read_value()? {
            hessian_rs::Value::Int(v) => visitor.visit_u8(v as u8),
            hessian_rs::Value::Long(v) => visitor.visit_u8(v as u8),
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(format!("deserialize u8 expect a int/long value, but get {}", v.to_string()).into()))),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        match self.de.read_value()? {
            hessian_rs::Value::Int(v) => visitor.visit_u16(v as u16),
            hessian_rs::Value::Long(v) => visitor.visit_u16(v as u16),
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(format!("deserialize u16 expect a int/long value, but get {}", v.to_string()).into()))),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        match self.de.read_value()? {
            hessian_rs::Value::Int(v) => visitor.visit_u32(v as u32),
            hessian_rs::Value::Long(v) => visitor.visit_u32(v as u32),
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(format!("deserialize u32 expect a int/long value, but get {}", v.to_string()).into()))),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        match self.de.read_value()? {
            hessian_rs::Value::Int(v) => visitor.visit_u64(v as u64),
            hessian_rs::Value::Long(v) => visitor.visit_u64(v as u64),
            v => Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(format!("deserialize u64 expect a int/long value, but get {}", v.to_string()).into()))),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        todo!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        todo!()
    }
}


#[cfg(test)]
mod tests {
    use crate::de::Decoder;
    use serde::de::Deserialize;

}