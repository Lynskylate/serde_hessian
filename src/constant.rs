pub enum ByteCodecType {
    ShortBinary(usize),
    Unknown,
}

impl ByteCodecType {
    pub fn from(c: u8) -> ByteCodecType {
        match c {
            0x20..=0x2f => ByteCodecType::ShortBinary((c - 0x20) as usize),
            _ => ByteCodecType::Unknown,
        }
    }
}
