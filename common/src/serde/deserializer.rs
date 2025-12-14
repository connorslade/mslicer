use std::{
    borrow::Cow,
    io::{ErrorKind, Read, Seek, SeekFrom},
    mem::{self, MaybeUninit},
    slice,
};

#[rustfmt::skip]
pub trait Deserializer {
    fn pos(&mut self) -> usize;
    fn size(&mut self) -> usize;
    fn advance_by(&mut self, amount: usize);
    fn jump_to(&mut self, pos: usize);
    fn execute_at<T>(&mut self, pos: usize, func: impl FnOnce(&mut Self) -> T) -> T;
    fn read_bytes(&mut self, length: usize) -> Cow<'_, [u8]>;
    fn is_eof(&mut self) -> bool;

    fn read_array<const LENGTH: usize>(&mut self) -> [u8; LENGTH] {
        self.read_bytes(LENGTH)
            .as_array::<LENGTH>()
            .copied()
            .unwrap_or([0; LENGTH])
    }

    fn read_bool(&mut self) -> bool { self.read_u8() != 0 }
    fn read_u8(&mut self) -> u8 { self.read_array::<1>()[0] }
    fn read_u16_be(&mut self) -> u16 { u16::from_be_bytes(self.read_array()) }
    fn read_u16_le(&mut self) -> u16 { u16::from_le_bytes(self.read_array()) }
    fn read_u32_be(&mut self) -> u32 { u32::from_be_bytes(self.read_array()) }
    fn read_u32_le(&mut self) -> u32 { u32::from_le_bytes(self.read_array()) }
    fn read_u64_be(&mut self) -> u64 { u64::from_be_bytes(self.read_array()) }
    fn read_u64_le(&mut self) -> u64 { u64::from_le_bytes(self.read_array()) }
    fn read_f32_be(&mut self) -> f32 { f32::from_be_bytes(self.read_array()) }
    fn read_f32_le(&mut self) -> f32 { f32::from_le_bytes(self.read_array()) }
    fn read_f64_be(&mut self) -> f64 { f64::from_be_bytes(self.read_array()) }
    fn read_f64_le(&mut self) -> f64 { f64::from_le_bytes(self.read_array()) }
}

pub struct SliceDeserializer<'a> {
    buffer: &'a [u8],
    offset: usize,
}

pub struct ReaderDeserializer<T: Read> {
    reader: T,
}

impl<'a> SliceDeserializer<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            buffer: data,
            offset: 0,
        }
    }

    pub fn read_slice(&mut self, length: usize) -> &'a [u8] {
        let value = &self.buffer[self.offset..self.offset + length];
        self.offset += length;
        value
    }
}

impl<T: Read> ReaderDeserializer<T> {
    pub fn new(reader: T) -> Self {
        Self { reader }
    }

    fn read_vec(&mut self, length: usize) -> Vec<u8> {
        let mut buf = Vec::<MaybeUninit<u8>>::with_capacity(length);
        let slice = unsafe { slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, length) };

        let mut written = 0;
        while written < length {
            match self.reader.read(&mut slice[written..]) {
                Ok(0) => break,
                Ok(n) => written += n,
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => panic!("failed to read from reader: {}", e),
            }
        }

        unsafe {
            buf.set_len(written);
            mem::transmute(buf)
        }
    }
}

impl<'a> Deserializer for SliceDeserializer<'a> {
    fn pos(&mut self) -> usize {
        self.offset
    }

    fn size(&mut self) -> usize {
        self.buffer.len()
    }

    fn advance_by(&mut self, amount: usize) {
        self.offset += amount;
    }

    fn jump_to(&mut self, pos: usize) {
        self.offset = pos;
    }

    fn execute_at<T>(&mut self, pos: usize, func: impl FnOnce(&mut Self) -> T) -> T {
        let offset = self.offset;
        self.jump_to(pos);
        let result = func(self);
        self.offset = offset;
        result
    }

    fn read_bytes(&mut self, length: usize) -> Cow<'_, [u8]> {
        let value = &self.buffer[self.offset..self.offset + length];
        self.offset += length;
        Cow::Borrowed(value)
    }

    fn is_eof(&mut self) -> bool {
        self.offset >= self.buffer.len()
    }
}

impl<T: Read + Seek> Deserializer for ReaderDeserializer<T> {
    fn pos(&mut self) -> usize {
        self.reader.stream_position().unwrap() as usize
    }

    fn size(&mut self) -> usize {
        self.reader.stream_len().unwrap() as usize
    }

    fn advance_by(&mut self, amount: usize) {
        self.reader.seek_relative(amount as i64).unwrap();
    }

    fn jump_to(&mut self, pos: usize) {
        self.reader.seek(SeekFrom::Start(pos as u64)).unwrap();
    }

    fn execute_at<K>(&mut self, pos: usize, func: impl FnOnce(&mut Self) -> K) -> K {
        let offset = self.reader.stream_position().unwrap();
        self.jump_to(pos);
        let result = func(self);
        self.jump_to(offset as usize);
        result
    }

    fn read_bytes(&mut self, length: usize) -> Cow<'_, [u8]> {
        Cow::Owned(self.read_vec(length))
    }

    fn is_eof(&mut self) -> bool {
        self.pos() >= self.size()
    }
}
