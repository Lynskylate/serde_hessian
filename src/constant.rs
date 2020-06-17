#[derive(Debug)]
pub enum Binary {
    ShortBinary(u8),
    TwoOctetBinary(u8),
    LongBinary(u8),
}

#[derive(Debug)]
pub enum Integer {
    DirectInt(u8),
    ByteInt(u8),
    ShortInt(u8),
    NormalInt,
}

#[derive(Debug)]
pub enum Long {
    DirectLong(u8),
    ByteLong(u8),
    ShortLong(u8),
    Int32Long,
    NormalLong,
}

#[derive(Debug)]
pub enum Date {
    Millisecond,
    Minute,
}

#[derive(Debug)]
pub enum ByteCodecType {
    True,
    False,
    Null,
    Definition,
    Int(Integer),
    Long(Long),
    Double(u8),
    Date(Date),
    Binary(Binary),
    // TODO: use enum to eliminate impossible states
    String(u8),
    Unknown,
}

impl ByteCodecType {
    #[inline]
    pub fn from(c: u8) -> ByteCodecType {
        match c {
            b'T' => ByteCodecType::True,
            b'F' => ByteCodecType::False,
            b'N' => ByteCodecType::Null,
            b'C' => ByteCodecType::Definition,
            // Integer
            0x80..=0xbf => ByteCodecType::Int(Integer::DirectInt(c)),
            0xc0..=0xcf => ByteCodecType::Int(Integer::ByteInt(c)),
            0xd0..=0xd7 => ByteCodecType::Int(Integer::ShortInt(c)),
            b'I' => ByteCodecType::Int(Integer::NormalInt),
            // Long
            0xd8..=0xef => ByteCodecType::Long(Long::DirectLong(c)),
            0xf0..=0xff => ByteCodecType::Long(Long::ByteLong(c)),
            0x38..=0x3f => ByteCodecType::Long(Long::ShortLong(c)),
            0x59 => ByteCodecType::Long(Long::Int32Long),
            b'L' => ByteCodecType::Long(Long::NormalLong),
            // Double
            b'D' | 0x5b..=0x5f => ByteCodecType::Double(c),
            // Date
            0x4a => ByteCodecType::Date(Date::Millisecond),
            0x4b => ByteCodecType::Date(Date::Minute),
            // Binary
            0x20..=0x2f => ByteCodecType::Binary(Binary::ShortBinary(c)),
            0x34..=0x37 => ByteCodecType::Binary(Binary::TwoOctetBinary(c)),
            b'B' | 0x41 => ByteCodecType::Binary(Binary::LongBinary(c)),
            // String
            0x00..=0x1f | 0x30..=0x33 | 0x52 | b'S' => ByteCodecType::String(c),
            _ => ByteCodecType::Unknown,
        }
    }
}
