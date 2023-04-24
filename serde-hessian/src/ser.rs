use crate::error::Error;
use hessian_rs::{ser::Serializer as ValueSerializer, value::Definition};

use serde::{
    ser::{self},
    Serialize,
};
use std::io;

type Result<T> = std::result::Result<T, Error>;

pub struct Serializer<W: io::Write>(ValueSerializer<W>);

impl<W: io::Write> Serializer<W> {
    pub fn new(writer: W) -> Self {
        Serializer(ValueSerializer::new(writer))
    }
}

pub struct StructSerializer<'a, W: io::Write> {
    name: &'static str,
    ser: &'a mut Serializer<W>,
    fields: Vec<&'a str>,
    inx: usize,
    buf: Vec<u8>,
}

pub struct MapSerializer<'a, W: io::Write> {
    name: Option<&'static str>,
    encoder: &'a mut Serializer<W>,
}

pub struct ListSerializer<'a, W: io::Write> {
    ser: &'a mut Serializer<W>,
    sized: bool,
}

impl<'a, W> StructSerializer<'a, W>
where
    W: io::Write,
{
    pub fn new(name: &'static str, ser: &'a mut Serializer<W>) -> Self {
        StructSerializer {
            name,
            ser,
            fields: Vec::new(),
            inx: 0,
            buf: Vec::new(),
        }
    }
}

impl<'a, W: io::Write> ser::SerializeStruct for StructSerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<U: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &U,
    ) -> Result<()> {
        if let Some(definition) = self.ser.0.get_definition(self.name) {
            if key != definition.fields[self.inx] {
                return Err(Error::SyntaxError(hessian_rs::ErrorKind::UnexpectedType(
                    "field name mismatch".to_string(),
                )));
            }
            self.inx += 1;
        } else {
            self.fields.push(key);
        }
        value.serialize(&mut Serializer::new(&mut self.buf))?;
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<()> {
        let def = match self.ser.0.get_definition(self.name) {
            Some(def) => def.clone(),
            None => {
                let def = Definition {
                    name: self.name.into(),
                    fields: self.fields.iter().map(|v| v.to_string()).collect(),
                };
                self.ser.0.write_definition(&def)?;
                def
            }
        };
        self.ser.0.write_object_start(&def)?;
        self.ser.0.extend_from_slice(&self.buf)?;
        Ok(())
    }
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
        if !self.sized {
            self.ser.0.write_object_end()?;
        }
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
        ser::SerializeTuple::end(self)
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
        self.ser.0.write_object_end()?;
        ser::SerializeTuple::end(self)?;
        Ok(())
    }
}

impl<'a, W: io::Write> ser::SerializeMap for MapSerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<()> {
        key.serialize(&mut *self.encoder)
    }

    #[inline]
    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<()> {
        value.serialize(&mut *self.encoder)
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
        key.serialize(&mut *self.encoder)?;
        value.serialize(&mut *self.encoder)
    }

    #[inline]
    fn end(self) -> Result<()> {
        self.encoder.0.write_object_end()?;
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
    fn end(self) -> Result<()> {
        self.encoder.0.write_object_end()?;
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
    fn end(self) -> Result<()> {
        self.encoder.0.write_object_end()?;
        // end of variant
        self.encoder.0.write_object_end()?;
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
    type SerializeStruct = StructSerializer<'a, W>;
    type SerializeStructVariant = MapSerializer<'a, W>;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<()> {
        self.0.serialize_bool(value)?;
        Ok(())
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<()> {
        self.0.serialize_int(value as i32)?;
        Ok(())
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<()> {
        self.0.serialize_int(value as i32)?;
        Ok(())
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<()> {
        self.0.serialize_int(value)?;
        Ok(())
    }

    #[inline]
    fn serialize_i64(self, value: i64) -> Result<()> {
        self.0.serialize_long(value)?;
        Ok(())
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<()> {
        self.0.serialize_int(value as i32)?;
        Ok(())
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<()> {
        self.0.serialize_int(value as i32)?;
        Ok(())
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<()> {
        if value < i32::max_value() as u32 {
            self.0.serialize_int(value as i32)?;
        } else {
            self.0.serialize_long(value as i64)?;
        }
        Ok(())
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<()> {
        self.0.serialize_long(value as i64)?;
        Ok(())
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<()> {
        self.0.serialize_double(value as f64)?;
        Ok(())
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<()> {
        self.0.serialize_double(value)?;
        Ok(())
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<()> {
        let mut buf = [0; 4];
        self.0.serialize_string(value.encode_utf8(&mut buf))?;
        Ok(())
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<()> {
        self.0.serialize_string(value)?;
        Ok(())
    }

    #[inline]
    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        self.0.serialize_binary(value)?;
        Ok(())
    }

    #[inline]
    fn serialize_unit(self) -> Result<()> {
        self.0.serialize_null()?;
        Ok(())
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.0.serialize_null()?;
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
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()> {
        self.0.write_map_start(Some(name))?;
        variant.serialize(&mut *self)?;
        value.serialize(&mut *self)?;
        self.0.write_object_end()?;
        Ok(())
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
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        match len {
            Some(len) => {
                self.0.write_list_begin(len, None)?;
                Ok(ListSerializer {
                    ser: self,
                    sized: true,
                })
            }
            None => Ok(ListSerializer {
                ser: self,
                sized: false,
            }),
        }
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.0.write_list_begin(len, None)?;
        Ok(ListSerializer {
            ser: self,
            sized: true,
        })
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.0.write_list_begin(len, Some(name))?;
        Ok(ListSerializer {
            ser: self,
            sized: true,
        })
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.0.write_map_start(Some(name))?;
        self.0.serialize_string(variant)?;
        self.0
            .write_list_begin(len, Some(&format!("{}.{}", name, variant)))?;
        Ok(ListSerializer {
            ser: self,
            sized: true,
        })
    }

    #[inline]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.0.write_map_start(None)?;
        Ok(MapSerializer {
            name: None,
            encoder: self,
        })
    }

    #[inline]
    fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(StructSerializer::new(name, self))
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.0.write_map_start(Some(name))?;
        self.serialize_str(variant)?;
        self.0.write_map_start(Some(variant))?;
        Ok(MapSerializer {
            name: Some(variant),
            encoder: self,
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
    use crate::ser::to_vec;
    use serde::Serialize;

    #[test]
    fn test_struct() {
        {
            #[derive(Serialize)]
            #[serde(rename = "example.Car")]
            struct Car {
                color: String,
                model: String,
            }
            let output = to_vec(&Car {
                color: "red".to_string(),
                model: "Ferrari".to_string(),
            })
            .unwrap();
            assert_eq!(
                output,
                &[
                    b'C', 0x0b, b'e', b'x', b'a', b'm', b'p', b'l', b'e', b'.', b'C', b'a', b'r',
                    0x92, 0x05, b'c', b'o', b'l', b'o', b'r', 0x05, b'm', b'o', b'd', b'e', b'l',
                    b'O', 0x90, 0x03, b'r', b'e', b'd', 0x07, b'F', b'e', b'r', b'r', b'a', b'r',
                    b'i',
                ]
            );
        }
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
                b'C', 0x04, b'T', b'e', b's', b't', 0x92, 0x03, b'i', b'n', b't', 0x03, b's', b'e',
                b'q', b'O', 0x90, 0x91, 0x7a, 0x01, b'a', 0x01, b'b'
            ]
        )
    }

    // todo: how keep consistence with java class?
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
        assert_eq!(
            to_vec(&n).unwrap(),
            &[b'M', 0x01, b'E', 0x07, b'N', b'e', b'w', b't', b'y', b'p', b'e', 0x91, b'Z']
        );

        // serialize tuple variant, use variant as list name
        let t = E::Tuple(1, 2);
        assert_eq!(
            to_vec(&t).unwrap(),
            &[
                b'M', 0x01, b'E', 0x05, b'T', b'u', b'p', b'l', b'e', 0x72, 0x07, b'E', b'.', b'T',
                b'u', b'p', b'l', b'e', 0x91, 0x92, b'Z'
            ]
        );

        // serialize Variant Struct, use variant naeme as map name
        let s = E::Struct { a: 1 };
        assert_eq!(
            to_vec(&s).unwrap(),
            &[
                b'M', 0x01, b'E', 0x06, b'S', b't', b'r', b'u', b'c', b't', b'M', 0x06, b'S', b't',
                b'r', b'u', b'c', b't', 0x01, b'a', 0x91, b'Z', b'Z'
            ]
        );
    }
}
