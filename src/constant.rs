
pub enum Binary {
    ShortBinary(u8),
    TwoOctetBinary(u8),
    LongBinary(u8),
}

pub enum Integer {
    DirectInt(u8),
    ByteInt(u8),
    ShortInt(u8),
    NormalInt,
}

pub enum ByteCodecType {
    True,
    False,
    Null,
    Int(Integer),
    Binary(Binary),
    Unknown,
}

impl ByteCodecType {
    #[inline]
    pub fn from(c: u8) -> ByteCodecType {
        match c {
            b'T' => ByteCodecType::True,
            b'F' => ByteCodecType::False,
            b'N' => ByteCodecType::Null,
            0x80..=0xbf => ByteCodecType::Int(Integer::DirectInt(c)),
            0xc0..=0xcf => ByteCodecType::Int(Integer::ByteInt(c)),
            0xd0..=0xd7 => ByteCodecType::Int(Integer::ShortInt(c)),
            b'I' => ByteCodecType::Int(Integer::NormalInt),
            0x20..=0x2f => ByteCodecType::Binary(Binary::ShortBinary(c)),
            0x34..=0x37 => ByteCodecType::Binary(Binary::TwoOctetBinary(c)),
            b'B' | b'A' => ByteCodecType::Binary(Binary::LongBinary(c)),
            _ => ByteCodecType::Unknown,
        }
    }
}
