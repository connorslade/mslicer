use anyhow::Result;

use common::serde::Deserializer;

use super::{Packet, QoS};
use crate::mqtt::misc::MqttDeserialize;

#[derive(Debug)]
pub struct SubscribePacket {
    pub packet_id: u16,
    pub filters: Vec<(String, QoS)>,
}

impl SubscribePacket {
    pub const PACKET_TYPE: u8 = 0x08;

    pub fn from_packet(packet: &Packet) -> Result<Self> {
        let mut des = Deserializer::new(&packet.remaining_bytes);

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
