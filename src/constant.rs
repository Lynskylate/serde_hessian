use std::fmt;

#[derive(Debug)]
pub enum Binary {
    Short(u8),
    TwoOctet(u8),
    Long(u8),
}

#[derive(Debug)]
pub enum Integer {
    Direct(u8),
    Byte(u8),
    Short(u8),
    Normal,
}

#[derive(Debug)]
pub enum Long {
    Direct(u8),
    Byte(u8),
    Short(u8),
    Int32,
    Normal,
}

#[derive(Debug)]
pub enum Double {
    Zero,
    One,
    Byte,
    Short,
    Float,
    Normal,
}

#[derive(Debug)]
pub enum Date {
    Millisecond,
    Minute,
}

#[derive(Debug)]
pub enum List {
    VarLength(bool /* typed */),
    FixedLength(bool /* typed */),
    ShortFixedLength(bool /* typed */, usize /* length */),
}

#[derive(Debug)]
pub enum String {
    /// string of length 0-31
    Compact(u8),
    /// string of length 0-1023
    Small(u8),
    /// non-final chunk
    Chunk,
    /// final chunk
    FinalChunk,
}

#[derive(Debug)]
pub enum Object {
    Compact(u8),
    Normal,
}

#[derive(Debug)]
pub enum ByteCodecType {
    True,
    False,
    Null,
    Definition,
    Object(Object),
    Ref,
    Int(Integer),
    Long(Long),
    Double(Double),
    Date(Date),
    Binary(Binary),
    List(List),
    Map(bool /*typed*/),
    String(String),
    Unknown,
}

impl ByteCodecType {
    #[inline]
    pub fn from(c: u8) -> ByteCodecType {
        match c {
            b'T' => ByteCodecType::True,
            b'F' => ByteCodecType::False,
            b'N' => ByteCodecType::Null,
            0x51 => ByteCodecType::Ref,
            // Map
            b'M' => ByteCodecType::Map(true),
            b'H' => ByteCodecType::Map(false),
            // List
            0x55 => ByteCodecType::List(List::VarLength(true)),
            b'V' => ByteCodecType::List(List::FixedLength(true)),
            0x57 => ByteCodecType::List(List::VarLength(false)),
            0x58 => ByteCodecType::List(List::FixedLength(false)),
            0x70..=0x77 => ByteCodecType::List(List::ShortFixedLength(true, (c - 0x70) as usize)),
            0x78..=0x7f => ByteCodecType::List(List::ShortFixedLength(false, (c - 0x78) as usize)),
            b'O' => ByteCodecType::Object(Object::Normal),
            0x60..=0x6f => ByteCodecType::Object(Object::Compact(c)),
            b'C' => ByteCodecType::Definition,
            // Integer
            0x80..=0xbf => ByteCodecType::Int(Integer::Direct(c)),
            0xc0..=0xcf => ByteCodecType::Int(Integer::Byte(c)),
            0xd0..=0xd7 => ByteCodecType::Int(Integer::Short(c)),
            b'I' => ByteCodecType::Int(Integer::Normal),
            // Long
            0xd8..=0xef => ByteCodecType::Long(Long::Direct(c)),
            0xf0..=0xff => ByteCodecType::Long(Long::Byte(c)),
            0x38..=0x3f => ByteCodecType::Long(Long::Short(c)),
            0x59 => ByteCodecType::Long(Long::Int32),
            b'L' => ByteCodecType::Long(Long::Normal),
            // Double
            0x5b => ByteCodecType::Double(Double::Zero),
            0x5c => ByteCodecType::Double(Double::One),
            0x5d => ByteCodecType::Double(Double::Byte),
            0x5e => ByteCodecType::Double(Double::Short),
            0x5f => ByteCodecType::Double(Double::Float),
            b'D' => ByteCodecType::Double(Double::Normal),
            // Date
            0x4a => ByteCodecType::Date(Date::Millisecond),
            0x4b => ByteCodecType::Date(Date::Minute),
            // Binary
            0x20..=0x2f => ByteCodecType::Binary(Binary::Short(c)),
            0x34..=0x37 => ByteCodecType::Binary(Binary::TwoOctet(c)),
            b'B' | 0x41 => ByteCodecType::Binary(Binary::Long(c)),
            // String
            // ::= [x00-x1f] <utf8-data>         # string of length 0-31
            0x00..=0x1f => ByteCodecType::String(String::Compact(c)),
            // ::= [x30-x34] <utf8-data>         # string of length 0-1023
            0x30..=0x33 => ByteCodecType::String(String::Small(c)),
            // x52 ('R') represents any non-final chunk
            0x52 => ByteCodecType::String(String::Chunk),
            // x53 ('S') represents the final chunk
            b'S' => ByteCodecType::String(String::FinalChunk),
            _ => ByteCodecType::Unknown,
        }
    }
}

impl fmt::Display for ByteCodecType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ByteCodecType::Int(_) => write!(f, "int"),
            ByteCodecType::Long(_) => write!(f, "long"),
            ByteCodecType::Double(_) => write!(f, "double"),
            ByteCodecType::Date(_) => write!(f, "date"),
            ByteCodecType::Binary(_) => write!(f, "binary"),
            ByteCodecType::String(_) => write!(f, "string"),
            ByteCodecType::List(_) => write!(f, "list"),
            ByteCodecType::Map(_) => write!(f, "map"),
            ByteCodecType::True | ByteCodecType::False => write!(f, "bool"),
            ByteCodecType::Null => write!(f, "null"),
            ByteCodecType::Definition => write!(f, "definition"),
            ByteCodecType::Ref => write!(f, "ref"),
            ByteCodecType::Object(_) => write!(f, "object"),
            ByteCodecType::Unknown => write!(f, "unknown"),
        }
    }
}
