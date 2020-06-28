use std::collections::HashMap;
use std::io::{self, Cursor, Read, Seek, SeekFrom};

use byteorder::{BigEndian, ReadBytesExt};

use super::constant::{Binary, ByteCodecType, Date, Double, Integer, List, Long};
use super::error::Error::SyntaxError;
use super::error::{ErrorKind, Result};
use super::value::{self, Defintion, Value};

pub struct Deserializer<R: AsRef<[u8]>> {
    buffer: Cursor<R>,
    type_references: Vec<String>,
    class_references: Vec<Defintion>,
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
                    return self.error(ErrorKind::UnexpectedType(v.to_string()));
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

    /// Read an object from buffer
    ///
    /// v2.0
    ///
    /// ```ignore
    /// class-def  ::= 'C(x43)' string int string*
    ///
    /// object     ::= 'O(x4f)' int value*
    ///            ::= [x60-x6f] value*
    /// ```
    ///
    /// See http://hessian.caucho.com/doc/hessian-serialization.html##object
    ///
    /// class definition:
    /// Hessian 2.0 has a compact object form where the field names are only serialized once.
    /// Following objects only need to serialize their values.
    ///
    /// The object definition includes a mandatory type string,
    /// the number of fields, and the field names.
    /// The object definition is stored in the object definition map
    /// and will be referenced by object instances with an integer reference.
    ///
    /// object instantiation:
    /// Hessian 2.0 has a compact object form where the field names are only serialized once.
    /// Following objects only need to serialize their values.
    ///
    /// The object instantiation creates a new object based on a previous definition.
    /// The integer value refers to the object definition.
    ///
    fn read_object(&mut self) -> Result<Value> {
        let val = self.read_value()?;
        if let Value::Int(i) = val {
            let definition = self
                .class_references
                .get(i as usize)
                .ok_or(SyntaxError(ErrorKind::OutOfDefinitionRange(i as usize)))?
                .clone();

            let mut map = HashMap::new();
            for k in definition.fields {
                let v = self.read_value()?;
                map.insert(Value::String(k), v);
            }
            Ok(Value::Map(map.clone().into()))
        } else {
            self.error(ErrorKind::UnexpectedType(val.to_string()))
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

    /// read bytes from buffer
    ///
    /// v2.0
    ///
    /// ```ignore
    /// binary ::= x41(A) b1 b0 <binary-data> binary
    ///        ::= x42(B) b1 b0 <binary-data>
    ///        ::= [x20-x2f] <binary-data>
    /// ```
    ///
    /// See http://hessian.caucho.com/doc/hessian-serialization.html##binary
    ///
    /// The octet x42 ('B') encodes the final chunk and
    /// x41 ('A') represents any non-final chunk.
    /// Each chunk has a 16-bit length value.
    ///
    /// len = 256/// b1 + b0
    ///
    /// Binary data with length less than 15 may be encoded by a single octet length [x20-x2f].
    ///
    /// len = code - 0x20
    ///
    fn read_binary(&mut self, bin: Binary) -> Result<Value> {
        match bin {
            Binary::Short(b) => Ok(Value::Bytes(self.read_bytes((b - 0x20) as usize)?)),
            Binary::TwoOctet(b) => {
                let second_byte = self.read_byte()?;
                let v = self.read_bytes(i16::from_be_bytes([b - 0x34, second_byte]) as usize)?;
                Ok(Value::Bytes(v))
            }
            Binary::Long(b) => self.read_long_binary(b),
        }
    }

    /// read a int from buffer
    ///
    /// v2.0
    ///
    /// ```ignore
    /// int ::= I(x49) b3 b2 b1 b0
    ///     ::= [x80-xbf]
    ///     ::= [xc0-xcf] b0
    ///     ::= [xd0-xd7] b1 b0
    /// ```
    ///
    /// See http://hessian.caucho.com/doc/hessian-serialization.html##int
    ///
    /// A 32-bit signed integer. An integer is represented by the octet x49 ('I')
    /// followed by the 4 octets of the integer in big-endian order.
    /// ```ignore
    /// value = (b3 << 24) + (b2 << 16) + (b1 << 8) + b0;
    /// ```
    ///
    /// single octet integers:
    /// Integers between -16 and 47 can be encoded by a single octet in the range x80 to xbf.
    /// ```ignore
    /// value = code - 0x90
    /// ```
    ///
    /// two octet integers:
    /// Integers between -2048 and 2047 can be encoded in two octets with the leading byte in the range xc0 to xcf.
    /// ```ignore
    /// value = ((code - 0xc8) << 8) + b0;
    /// ```
    ///
    /// three octet integers:
    /// Integers between -262144 and 262143 can be encoded in three bytes with the leading byte in the range xd0 to xd7.
    /// ```ignore
    /// value = ((code - 0xd4) << 16) + (b1 << 8) + b0;
    /// ```
    ///
    fn read_int(&mut self, i: Integer) -> Result<Value> {
        match i {
            Integer::Direct(b) => Ok(Value::Int(b as i32 - 0x90)),
            Integer::Byte(b) => {
                let b2 = self.read_byte()?;
                Ok(Value::Int(
                    i16::from_be_bytes([b.overflowing_sub(0xc8).0, b2]) as i32,
                ))
            }
            Integer::Short(b) => {
                let bs = self.read_bytes(2)?;
                Ok(Value::Int(
                    i32::from_be_bytes([b.overflowing_sub(0xd4).0, bs[0], bs[1], 0x00]) >> 8,
                ))
            }
            Integer::Normal => {
                let val = self.buffer.read_i32::<BigEndian>()?;
                Ok(Value::Int(val))
            }
        }
    }

    /// read a long from buffer
    ///
    /// v2.0
    /// ```ignore
    /// long ::= L(x4c) b7 b6 b5 b4 b3 b2 b1 b0
    ///      ::= [xd8-xef]
    ///      ::= [xf0-xff] b0
    ///      ::= [x38-x3f] b1 b0
    ///      ::= x4c b3 b2 b1 b0
    /// ```
    ///
    /// See http://hessian.caucho.com/doc/hessian-serialization.html##long
    ///
    /// A 64-bit signed integer. An long is represented by the octet x4c ('L' )
    /// followed by the 8-bytes of the integer in big-endian order.
    ///
    /// single octet longs:
    /// Longs between -8 and 15 are represented by a single octet in the range xd8 to xef.
    /// ```ignore
    /// value = (code - 0xe0)
    /// ```
    ///
    /// two octet longs:
    /// Longs between -2048 and 2047 are encoded in two octets with the leading byte in the range xf0 to xff.
    /// ```ignore
    /// value = ((code - 0xf8) << 8) + b0
    /// ```
    ///
    /// three octet longs:
    /// Longs between -262144 and 262143 are encoded in three octets with the leading byte in the range x38 to x3f.
    /// ```ignore
    /// value = ((code - 0x3c) << 16) + (b1 << 8) + b0
    /// ```
    ///
    /// four octet longs:
    /// Longs between which fit into 32-bits are encoded in five octets with the leading byte x59.
    /// ```ignore
    /// value = (b3 << 24) + (b2 << 16) + (b1 << 8) + b0
    /// ```
    ///
    fn read_long(&mut self, l: Long) -> Result<Value> {
        match l {
            Long::Direct(b) => Ok(Value::Long(b as i64 - 0xe0)),
            Long::Byte(b) => {
                let b2 = self.read_byte()?;
                Ok(Value::Long(
                    i16::from_be_bytes([b.overflowing_sub(0xf8).0, b2]) as i64,
                ))
            }
            Long::Short(b) => {
                let bs = self.read_bytes(2)?;
                Ok(Value::Long(
                    (i32::from_be_bytes([b.overflowing_sub(0x3c).0, bs[0], bs[1], 0x00]) >> 8)
                        as i64,
                ))
            }
            Long::Int32 => Ok(Value::Long(self.buffer.read_i32::<BigEndian>()? as i64)),
            Long::Normal => Ok(Value::Long(self.buffer.read_i64::<BigEndian>()?)),
        }
    }

    /// read a double from buffer
    ///
    /// v2.0
    /// ```ignore
    /// double ::= D(x44) b7 b6 b5 b4 b3 b2 b1 b0
    ///        ::= x5b
    ///        ::= x5c
    ///        ::= x5d(byte) b0
    ///        ::= x5e(short) b1 b0
    ///        ::= x5f(float) b3 b2 b1 b0
    /// ```
    ///
    /// See http://hessian.caucho.com/doc/hessian-serialization.html##double
    ///
    /// The double 0.0 can be represented by the octet x5b
    /// The double 1.0 can be represented by the octet x5c
    ///
    /// double octet:
    /// Doubles between -128.0 and 127.0 with no fractional component
    /// can be represented in two octets by casting the byte value to a double.
    /// ```ignore
    /// value = (double) b0
    /// ```
    ///
    /// double short:
    /// Doubles between -32768.0 (-0x8000) and 32767.0(0x8000 - 1) with no fractional component
    /// can be represented in three octets by casting the short value to a double.
    /// ```ignore
    /// value = (double) (256/// b1 + b0)
    /// ```
    ///
    /// double float:
    /// Doubles which are equivalent to their 32-bit float representation
    /// can be represented as the 4-octet float and then cast to double.
    ///
    fn read_double(&mut self, tag: Double) -> Result<Value> {
        let val = match tag {
            Double::Normal => self.buffer.read_f64::<BigEndian>()?,
            Double::Zero => 0.0,
            Double::One => 1.0,
            Double::Byte => self.buffer.read_i8()? as f64,
            Double::Short => self.buffer.read_i16::<BigEndian>()? as f64,
            Double::Float => (self.buffer.read_i32::<BigEndian>()? as f64) * 0.001,
        };
        Ok(Value::Double(val))
    }

    /// read a date from buffer,
    ///
    /// v2.0
    /// ```ignore
    /// date ::= x4a(J) b7 b6 b5 b4 b3 b2 b1 b0 // Date represented by a 64-bit long of milliseconds since Jan 1 1970 00:00H, UTC.
    ///      ::= x4b(K) b4 b3 b2 b1 b0          // The second form contains a 32-bit int of minutes since Jan 1 1970 00:00H, UTC.
    /// ```
    ///
    /// See http://hessian.caucho.com/doc/hessian-serialization.html##date
    ///
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

    /// read a string from buffer
    ///
    /// The length is the number of characters, which may be different than the number of bytes.
    ///
    /// v2.0
    /// ```ignore
    /// string ::= R(x52) b1 b0 <utf8-data> string  # non-final chunk
    ///        ::= S(x53) b1 b0 <utf8-data>         # string of length 0-65535
    ///        ::= [x00-x1f] <utf8-data>            # string of length 0-31
    ///        ::= [x30-x33] b0 <utf8-data>         # string of length 0-1023
    /// ```
    ///
    /// See http://hessian.caucho.com/doc/hessian-serialization.html##string
    ///
    /// A 16-bit unicode character string encoded in UTF-8. Strings are encoded in chunks.
    /// x53 ('S') represents the final chunk and x52 ('R') represents any non-final chunk.
    /// Each chunk has a 16-bit unsigned integer length value.
    ///
    /// The length is the number of 16-bit characters, which may be different than the number of bytes.
    /// String chunks may not split surrogate pairs.
    ///
    /// short strings:
    /// Strings with length less than 32 may be encoded with a single octet length [x00-x1f].
    /// ```ignore
    /// [x00-x1f] <utf8-data>
    /// ```
    ///
    fn read_string(&mut self, tag: u8) -> Result<Value> {
        let buf = self.read_string_internal(tag)?;
        let s = unsafe { String::from_utf8_unchecked(buf) };
        Ok(Value::String(s))
    }

    /// v2.0
    /// ```ignore
    /// ref ::= (0x51) int(putInt)
    /// ```
    ///
    /// See http://hessian.caucho.com/doc/hessian-serialization.html##ref
    ///
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
                    self.error(ErrorKind::OutOfTypeRefRange(i as usize))
                }
            }
            Ok(v) => self.error(ErrorKind::UnexpectedType(v.to_string())),
            Err(e) => Err(e),
        }
    }

    fn read_varlength_map_internal(&mut self) -> Result<HashMap<Value, Value>> {
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

    /// read an array from buffer
    ///
    /// v2.0
    /// ```ignore
    /// list ::= x55 type value* 'Z'   # variable-length list
    ///      ::= 'V(x56)' type int value*   # fixed-length list
    ///      ::= x57 value* 'Z'        # variable-length untyped list
    ///      ::= x58 int value*        # fixed-length untyped list
    ///      ::= [x70-77] type value*  # fixed-length typed list
    ///      ::= [x78-7f] value*       # fixed-length untyped list
    /// ```
    ///
    /// See http://hessian.caucho.com/doc/hessian-serialization.html##list
    ///
    /// An ordered list, like an array.
    /// The two list productions are a fixed-length list and a variable length list.
    /// Both lists have a type.
    /// The type string may be an arbitrary UTF-8 string understood by the service.
    ///
    /// fixed length list:
    /// Hessian 2.0 allows a compact form of the list for successive lists of
    /// the same type where the length is known beforehand.
    /// The type and length are encoded by integers,
    /// where the type is a reference to an earlier specified type.
    ///
    fn read_list(&mut self, list: List) -> Result<Value> {
        // TODO(lynskylate@gmail.com): Should add list to reference, but i don't know any good way to deal with it
        match list {
            List::ShortFixedLength(typed, length) => {
                let list = if typed {
                    let typ = self.read_type()?;
                    let val = self.read_exact_length_list_internal(length)?;
                    value::List::from((typ, val))
                } else {
                    let val = self.read_exact_length_list_internal(length)?;
                    value::List::from(val)
                };
                Ok(Value::List(list))
            }
            List::VarLength(typed) => {
                let list = if typed {
                    let typ = self.read_type()?;
                    let val = self.read_varlength_list_internal()?;
                    value::List::from((typ, val))
                } else {
                    let val = self.read_varlength_list_internal()?;
                    value::List::from(val)
                };
                Ok(Value::List(list))
            }
            List::FixedLength(typed) => {
                let list = if typed {
                    let typ = self.read_type()?;
                    let length = match self.read_value()? {
                        Value::Int(l) => l as usize,
                        v @ _ => return self.error(ErrorKind::UnexpectedType(v.to_string())),
                    };
                    let val = self.read_exact_length_list_internal(length)?;
                    value::List::from((typ, val))
                } else {
                    let length = match self.read_value()? {
                        Value::Int(l) => l as usize,
                        v @ _ => return self.error(ErrorKind::UnexpectedType(v.to_string())),
                    };
                    let val = self.read_exact_length_list_internal(length)?;
                    value::List::from(val)
                };
                Ok(Value::List(list))
            }
        }
    }

    /// read an map from buffer
    ///
    /// v2.0
    /// ```ignore
    /// map        ::= 'M' type (value value)* 'Z'  # key, value map pairs
    ///            ::= 'H' (value value)* 'Z'       # untyped key, value
    /// ```
    ///
    /// See http://hessian.caucho.com/doc/hessian-serialization.html##map
    ///
    /// Represents serialized maps and can represent objects.
    /// The type element describes the type of the map.
    /// The type may be empty, i.e. a zero length.
    /// The parser is responsible for choosing a type if one is not specified.
    /// For objects, unrecognized keys will be ignored.
    ///
    /// Each map is added to the reference list. Any time the parser expects a map,
    /// it must also be able to support a null or a ref.
    ///
    /// The type is chosen by the service.
    ///
    fn read_map(&mut self, typed: bool) -> Result<Value> {
        let map = if typed {
            let typ = self.read_type()?;
            value::Map::from((typ, self.read_varlength_map_internal()?))
        } else {
            value::Map::from(self.read_varlength_map_internal()?)
        };
        Ok(Value::Map(map))
    }

    /// v2.0
    /// ```ignore
    /// ref ::= Q(x51) int
    /// ```
    ///
    /// See http://hessian.caucho.com/doc/hessian-serialization.html##ref
    ///
    /// Each map or list is stored into an array as it is parsed.
    /// ref selects one of the stored objects. The first object is numbered '0'.
    ///
    fn read_ref(&mut self) -> Result<Value> {
        match self.read_value()? {
            Value::Int(i) => Ok(Value::Ref(i as u32)),
            v @ _ => self.error(ErrorKind::UnexpectedType(v.to_string())),
        }
    }

    /// Read a hessian 2.0 value
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
            }
            ByteCodecType::Ref => self.read_ref(),
            ByteCodecType::Object => self.read_object(),
            _ => self.error(ErrorKind::UnknownType),
        }
    }
}

/// Read a hessain 2.0 value from a slice
pub fn from_slice(v: &[u8]) -> Result<Value> {
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
        // FIXME: should the type be `int` or `[int`?
        test_decode_ok(
            &[b'V', 0x04, b'[', b'i', b'n', b't', 0x92, 0x90, 0x91],
            Value::List(("[int", vec![Value::Int(0), Value::Int(1)]).into()),
        );
        //Untyped variable list
        test_decode_ok(
            &[0x57, 0x90, 0x91, b'Z'],
            Value::List(vec![Value::Int(0), Value::Int(1)].into()),
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
            Value::Map(("com.caucho.test.car", map.clone()).into()),
        );

        test_decode_ok(
            &[
                b'H', 0x91, 0x03, b'f', b'e', b'e', 0xa0, 0x03, b'f', b'i', b'e', 0xc9, 0x00, 0x03,
                b'f', b'o', b'e', b'Z',
            ],
            Value::Map(map.clone().into()),
        );
    }

    #[test]
    fn test_read_object() {
        let mut map = HashMap::new();
        map.insert(
            Value::String("Color".to_string()),
            Value::String("red".to_string()),
        );
        map.insert(
            Value::String("Model".to_string()),
            Value::String("corvette".to_string()),
        );
        test_decode_ok(
            &[
                b'C', 0x0b, b'e', b'x', b'a', b'm', b'p', b'l', b'e', b'.', b'C', b'a', b'r', 0x92,
                0x05, b'C', b'o', b'l', b'o', b'r', 0x05, b'M', b'o', b'd', b'e', b'l', b'O', 0x90,
                0x03, b'r', b'e', b'd', 0x08, b'c', b'o', b'r', b'v', b'e', b't', b't', b'e',
            ],
            Value::Map(map.clone().into()),
        );
    }

    #[test]
    fn test_read_ref() {
        let mut map = HashMap::new();
        map.insert(Value::String("head".to_string()), Value::Int(1));
        map.insert(Value::String("tail".to_string()), Value::Ref(0));
        test_decode_ok(
            &[
                b'C', 0x0a, b'L', b'i', b'n', b'k', b'e', b'd', b'L', b'i', b's', b't', 0x92, 0x04,
                b'h', b'e', b'a', b'd', 0x04, b't', b'a', b'i', b'l', b'O', 0x90, 0x91, 0x51, 0x90,
            ],
            Value::Map(map.clone().into()),
        );
    }
}
