use anyhow::Result;
use bitflags::bitflags;

use common::serde::{Deserializer, DynamicSerializer, Serializer};

use crate::mqtt::misc::{MqttDeserialize, MqttSerializer};

use super::Packet;

#[derive(Debug)]
pub struct PublishPacket {
    pub flags: PublishFlags,
    pub topic: String,
    pub packet_id: Option<u16>,
    pub data: Vec<u8>,
}

bitflags! {
    #[derive(Debug)]
    pub struct PublishFlags: u8 {
        const DUP = 0b1000;
        const QOS1 = 0b0010;
        const QOS2 = 0b0100;
        const RETAIN = 0b0001;
    }
}

impl PublishPacket {
    pub const PACKET_TYPE: u8 = 0x03;

    pub fn from_packet(packet: &Packet) -> Result<Self> {
        assert_eq!(packet.packet_type, Self::PACKET_TYPE);
        let mut des = Deserializer::new(&packet.remaining_bytes);

        let flags = PublishFlags::from_bits(packet.flags).unwrap();
        let topic = des.read_string().into_owned();

        let packet_id = (flags.contains(PublishFlags::QOS1) || flags.contains(PublishFlags::QOS2))
            .then(|| des.read_u16());
        let data = des
            .read_bytes(packet.remaining_length as usize - des.pos())
            .to_vec();

        Ok(Self {
            flags,
            topic,
            packet_id,
            data,
        })
    }

    pub fn to_packet(&self) -> Packet {
        let mut ser = DynamicSerializer::new();
        ser.write_string(&self.topic);
        ser.write_bytes(&self.data);

        let data = ser.into_inner();
        Packet {
            packet_type: Self::PACKET_TYPE,
            flags: self.flags.bits(),
            remaining_length: data.len() as u32,
            remaining_bytes: data,
        }
    }
}
