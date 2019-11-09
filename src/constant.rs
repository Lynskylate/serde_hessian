
pub enum ByteCodecType{
    ShortBinary(usize),
    Unknown,
}


impl ByteCodecType{
    pub fn from(c: u8)->ByteCodecType{
        if c >= 0x20 || c <= 0x2f {
            return ByteCodecType::ShortBinary((c - 0x20) as usize)
        }
        ByteCodecType::Unknown
    }
}
