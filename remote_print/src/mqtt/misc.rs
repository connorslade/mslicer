use std::{
    borrow::Cow,
    sync::atomic::{AtomicU64, Ordering},
};

use common::serde::{Deserializer, Serializer, SliceDeserializer};

pub trait MqttDeserialize<'a> {
    fn read_string(&mut self) -> Cow<'a, str>;
}

pub trait MqttSerializer {
    fn write_string(&mut self, data: &str);
}

impl<'a> MqttDeserialize<'a> for SliceDeserializer<'a> {
    fn read_string(&mut self) -> Cow<'a, str> {
        let len = self.read_u16_be();
        let buf = self.read_slice(len as usize);
        String::from_utf8_lossy(buf)
    }
}

impl<T> MqttSerializer for T
where
    T: Serializer,
{
    fn write_string(&mut self, data: &str) {
        assert!(data.len() <= u16::MAX as usize);
        let len = data.len() as u16;
        self.write_u16_be(len);
        self.write_bytes(data.as_bytes());
    }
}

pub fn next_id() -> u64 {
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}
