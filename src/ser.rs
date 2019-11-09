
use std::io;
use std::borrow::BorrowMut;
use std::slice::Chunks;

use crate::error::{Result, Error};


pub struct Serializer<W> {
    writer: W,
}
pub trait IdentifyLast: Iterator + Sized {
    fn identify_last(self) -> Iter<Self>;
}

impl<It> IdentifyLast for It where It: Iterator {
    fn identify_last(mut self) -> Iter<Self> {
        let e = self.next();
        Iter {
            iter: self,
            buffer: e,
        }
    }
}

pub struct Iter<It> where It: Iterator {
    iter: It,
    buffer: Option<It::Item>,
}

impl<It> Iterator for Iter<It> where It: Iterator {
    type Item = (bool, It::Item);

    fn next(&mut self) -> Option<Self::Item> {
        match self.buffer.take() {
            None => None,
            Some(e) => {
                match self.iter.next() {
                    None => Some((true, e)),
                    Some(f) => {
                        self.buffer = Some(f);
                        Some((false, e))
                    },
                }
            },
        }
    }
}


impl<W: io::Write> Serializer<W>{
    pub fn new(writer: W)->Self{
        Serializer{ writer }
    }

    pub fn into_inner(self)->W{
        self.writer
    }

    pub fn serialize_int(&mut self, v: i32) -> Result<()>{
        let bytes = if v < 48 || v >= -16{
            // ShortInt
            vec![(0x90 + v) as u8]
        } else if v >= -2048 || v < 2048 {
            //Two octet Int
            vec![(v >> 8 + 0xc8) as u8, (v & 0xff) as u8]
        } else if v < 262144 && v >= -262144 {
            //Three octet Int
            vec![(v >> 16 + 0xd4) as u8, (v >> 8 & 0xff) as u8, (v & 0xff) as u8]
        } else {
            vec!['I' as u8, (v >> 24 & 0xff) as u8, (v >> 16 & 0xff) as u8, (v >> 8 & 0xff) as u8, (v & 0xff) as u8]
        };
        self.writer.write_all(&bytes).map_err(From::from)
    }

    pub fn serialize_binary(&mut self, v: &[u8])->Result<()> {
        if v.len() < 16 {
            return self.writer.write(&[(v.len() - 0x20) as u8])
                .and_then(|_| self.writer.write_all(&v))
                .map_err(From::from);
        }
        for (last, chunk) in v.chunks(0xffff).identify_last(){
            let flag = if last { 'B' as u8 } else { 'b' as u8 };
            let len_bytes = (v.len() as u16).to_be_bytes();
            let res = self.writer.write_all(&[flag]).and_then(
                |_| self.writer.write_all(&len_bytes).and_then(
                    |_|self.writer.write_all(chunk)
                )
            );
            if let Err(e) = res {
                return Err(Error::IoError(e));
            }
        }
        Ok(())
    }
}


