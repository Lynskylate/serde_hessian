use crate::value::Value::Bytes;

pub enum Binary {
    ShortBinary(u8),
    TwoOctetBinary(u8),
    LongBinary(u8),
}

pub enum ByteCodecType {
    True,
    False,
    Null,
    Binary(Binary),
    Unknown,
}

impl ByteCodecType {
    pub fn from(c: u8) -> ByteCodecType {
        match c {
            b'T' => ByteCodecType::True,
            b'F' => ByteCodecType::False,
            b'N' => ByteCodecType::Null,
            0x20..=0x2f => ByteCodecType::Binary(Binary::ShortBinary(c)),
            0x34..=0x37 => ByteCodecType::Binary(Binary::TwoOctetBinary(c)),
            b'B' | b'A' => ByteCodecType::Binary(Binary::LongBinary(c)),
            _ => ByteCodecType::Unknown,
        }
    }
}
