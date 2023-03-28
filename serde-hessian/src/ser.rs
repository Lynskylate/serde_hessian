use crate::error::Error;
use hessian_rs::{
    ser::Serializer as ValueSerializer,
    value::{List, Map},
};
use serde::ser::SerializeMap;
use serde::ser::SerializeSeq;
use serde::ser::SerializeStruct;
use serde::ser::SerializeStructVariant;
use serde::{ser, Serialize};
use std::io;

type Result<T> = std::result::Result<T, Error>;

pub struct Serializer<W: io::Write> {
    ser: ValueSerializer<W>,
}

impl<W: io::Write> Serializer<W> {
    pub fn new(writer: W) -> Self {
        Serializer {
            ser: ValueSerializer::new(writer),
        }
    }
}

pub struct StructSerializer<W: io::Write> {
    name: &'static str,
    ser: Serializer<W>,
}

pub struct MapSerializer<'a, W: io::Write> {
    name: Option<&'static str>,
    ser: &'a mut Serializer<W>,
}

pub struct ListSerializer<'a, W: io::Write> {
    ser: &'a mut Serializer<W>,
    sized: bool,
}

impl<'a, W: io::Write> ser::SerializeSeq for ListSerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<()> {
        value.serialize(&mut *self.ser)?;
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: io::Write> ser::SerializeTuple for ListSerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<()> {
        value.serialize(&mut *self.ser)?;
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: io::Write> ser::SerializeTupleStruct for ListSerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<()> {
        ser::SerializeTuple::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: io::Write> ser::SerializeTupleVariant for ListSerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<()> {
        ser::SerializeTuple::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: io::Write> ser::SerializeMap for MapSerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<()> {
        key.serialize(&mut *self.ser)
    }

    #[inline]
    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<()> {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn serialize_entry<K: ?Sized, V: ?Sized>(
        &mut self,
        key: &K,
        value: &V,
    ) -> std::result::Result<(), Self::Error>
    where
        K: Serialize,
        V: Serialize,
    {
        key.serialize(&mut *self.ser)?;
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        self.ser.ser.write_object_end()?;
        Ok(())
    }
}

impl<'a, W: io::Write> ser::SerializeStruct for MapSerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        ser::SerializeMap::serialize_key(self, key)?;
        ser::SerializeMap::serialize_value(self, value)
    }

    #[inline]
    fn end(mut self) -> Result<()> {
        self.ser.ser.write_object_end()?;
        Ok(())
    }
}

impl<'a, W: io::Write> ser::SerializeStructVariant for MapSerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        ser::SerializeMap::serialize_key(self, key)?;
        ser::SerializeMap::serialize_value(self, value)
    }

    #[inline]
    fn end(mut self) -> Result<()> {
        self.ser.ser.write_object_end()?;
        Ok(())
    }
}

impl<'a, W: io::Write> ser::Serializer for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = ListSerializer<'a, W>;
    type SerializeTuple = Self::SerializeSeq;
    type SerializeTupleStruct = Self::SerializeTuple;
    type SerializeTupleVariant = Self::SerializeTuple;
    type SerializeMap = MapSerializer<'a, W>;
    type SerializeStruct = Self::SerializeMap;
    type SerializeStructVariant = Self::SerializeStruct;

    #[inline]
    fn serialize_bool(mut self, value: bool) -> Result<()> {
        self.ser.serialize_bool(value)?;
        Ok(())
    }

    #[inline]
    fn serialize_i8(mut self, value: i8) -> Result<()> {
        self.ser.serialize_int(value as i32)?;
        Ok(())
    }

    #[inline]
    fn serialize_i16(mut self, value: i16) -> Result<()> {
        self.ser.serialize_int(value as i32)?;
        Ok(())
    }

    #[inline]
    fn serialize_i32(mut self, value: i32) -> Result<()> {
        self.ser.serialize_int(value)?;
        Ok(())
    }

    #[inline]
    fn serialize_i64(mut self, value: i64) -> Result<()> {
        self.ser.serialize_long(value)?;
        Ok(())
    }

    #[inline]
    fn serialize_u8(mut self, value: u8) -> Result<()> {
        self.ser.serialize_int(value as i32)?;
        Ok(())
    }

    #[inline]
    fn serialize_u16(mut self, value: u16) -> Result<()> {
        self.ser.serialize_int(value as i32)?;
        Ok(())
    }

    #[inline]
    fn serialize_u32(mut self, value: u32) -> Result<()> {
        if value < i32::max_value() as u32 {
            self.ser.serialize_int(value as i32)?;
        } else {
            self.ser.serialize_long(value as i64)?;
        }
        Ok(())
    }

    #[inline]
    fn serialize_u64(mut self, value: u64) -> Result<()> {
        self.ser.serialize_long(value as i64)?;
        Ok(())
    }

    #[inline]
    fn serialize_f32(mut self, value: f32) -> Result<()> {
        self.ser.serialize_double(value as f64)?;
        Ok(())
    }

    #[inline]
    fn serialize_f64(mut self, value: f64) -> Result<()> {
        self.ser.serialize_double(value as f64)?;
        Ok(())
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<()> {
        let mut buf = [0; 4];
        self.ser.serialize_string(value.encode_utf8(&mut buf))?;
        Ok(())
    }

    #[inline]
    fn serialize_str(mut self, value: &str) -> Result<()> {
        self.ser.serialize_string(value)?;
        Ok(())
    }

    #[inline]
    fn serialize_bytes(mut self, value: &[u8]) -> Result<()> {
        self.ser.serialize_binary(value)?;
        Ok(())
    }

    #[inline]
    fn serialize_unit(mut self) -> Result<()> {
        self.ser.serialize_null()?;
        Ok(())
    }

    #[inline]
    fn serialize_unit_struct(mut self, _name: &'static str) -> Result<()> {
        self.ser.serialize_null()?;
        Ok(())
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<()> {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()> {
        value.serialize(self)
    }

    #[inline]
    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<()> {
        value.serialize(self)
    }

    #[inline]
    fn serialize_seq(mut self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        match len {
            Some(len) => {
                self.ser.write_list_begin(len, None)?;
                Ok(ListSerializer { ser: self, sized: true })
            }
            None => {
                Ok(ListSerializer { ser: self, sized: false})
            }
        }
    }

    #[inline]
    fn serialize_tuple(mut self, len: usize) -> Result<Self::SerializeTuple> {
        self.ser.write_list_begin(len, None)?;
        Ok(ListSerializer { ser: self, sized: true })
    }

    #[inline]
    fn serialize_tuple_struct(
        mut self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.ser.write_list_begin(len, Some(name))?;
        Ok(ListSerializer { ser: self, sized: true})
    }

    #[inline]
    fn serialize_tuple_variant(
        mut self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.ser.write_list_begin(len, Some(variant))?;
        Ok(ListSerializer { ser: self , sized: true})
    }

    #[inline]
    fn serialize_map(mut self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.ser.write_map_start(None)?;
        Ok(MapSerializer {
            name: None,
            ser: self,
        })
    }

    #[inline]
    fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        // TODO: Use definition + object replace map
        self.ser.write_map_start(Some(name))?;
        Ok(MapSerializer {
            name: Some(name),
            ser: self,
        })
    }

    #[inline]
    fn serialize_struct_variant(
        mut self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.ser.write_map_start(Some(variant))?;
        Ok(MapSerializer {
            name: Some(name),
            ser: self,
        })
    }

    fn serialize_i128(self, v: i128) -> std::result::Result<Self::Ok, Self::Error> {
        let _ = v;
        Err(ser::Error::custom("i128 is not supported"))
    }

    fn serialize_u128(self, v: u128) -> std::result::Result<Self::Ok, Self::Error> {
        let _ = v;
        Err(ser::Error::custom("u128 is not supported"))
    }

    fn collect_str<T: ?Sized>(self, value: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: std::fmt::Display,
    {
        self.serialize_str(&value.to_string())
    }

    fn is_human_readable(&self) -> bool {
        false
    }



}

pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut buf = Vec::new();
    let mut ser = Serializer::new(&mut buf);
    value.serialize(&mut ser)?;
    Ok(buf)
}

#[cfg(test)]
mod test {
    use serde::Serialize;
    use crate::ser::to_vec;

    #[test]
    fn test_struct() {
        #[derive(Serialize)]
        struct Test {
            int: u32,
            seq: Vec<&'static str>,
        }

        let test = Test {
            int: 1,
            seq: vec!["a", "b"],
        };
        let output = to_vec(&test).unwrap();
        assert_eq!(
            output,
            &[
                b'M', 0x04, b'T', b'e', b's', b't', 0x03, b'i', b'n', b't', 0x91, 0x03, b's', b'e',
                b'q', 0x7a, 0x01, b'a', 0x01, b'b', b'Z'
            ]
        )
    }
    #[test]
    fn test_enum() {
        #[derive(Serialize)]
        enum E {
            Unit,
            Newtype(u32),
            Tuple(u32, u32),
            Struct { a: u32 },
        }

        let u = E::Unit;
        let expected = b"\x04Unit";
        assert_eq!(to_vec(&u).unwrap(), expected);

        let n = E::Newtype(1);
        assert_eq!(to_vec(&n).unwrap(), &[0x91]);

        // serialize tuple variant, use variant as list name
        let t = E::Tuple(1, 2);
        assert_eq!(
            to_vec(&t).unwrap(),
            &[0x72, 0x05, b'T', b'u', b'p', b'l', b'e', 0x91, 0x92]
        );

        // serialize Variant Struct, use variant naeme as map name
        let s = E::Struct { a: 1 };
        assert_eq!(
            to_vec(&s).unwrap(),
            &[b'M', 0x06, b'S', b't', b'r', b'u', b'c', b't', 0x01, b'a', 0x91, b'Z']
        );
    }
}
