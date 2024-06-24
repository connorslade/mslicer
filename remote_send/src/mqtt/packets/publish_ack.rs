use common::serde::{DynamicSerializer, Serializer};

use super::Packet;

pub struct PublishAckPacket {
    pub packet_id: u16,
}

impl PublishAckPacket {
    pub const PACKET_TYPE: u8 = 0x04;

    pub fn to_packet(&self) -> Packet {
        let mut ser = DynamicSerializer::new();
        ser.write_u16(self.packet_id);

        let data = ser.into_inner();
        Packet {
            packet_type: Self::PACKET_TYPE,
            flags: 0,
            remaining_length: data.len() as u32,
            remaining_bytes: data,
        }
    }
}
