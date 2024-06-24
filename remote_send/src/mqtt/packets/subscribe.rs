use anyhow::Result;

use common::serde::Deserializer;

use crate::mqtt::MqttDeserialize;

#[derive(Debug)]
pub struct SubscribePacket {
    pub packet_id: u16,
    pub filters: Vec<(String, QoS)>,
}

#[derive(Debug)]
pub struct QoS(pub u8);

impl SubscribePacket {
    pub const PACKET_TYPE: u8 = 0x08;

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut des = Deserializer::new(bytes);

        let packet_id = des.read_u16();
        let mut filters = Vec::new();
        while !des.is_empty() {
            let topic = des.read_string().into_owned();
            let qos = des.read_u8();
            filters.push((topic, QoS(qos)));
        }

        Ok(Self { packet_id, filters })
    }
}
