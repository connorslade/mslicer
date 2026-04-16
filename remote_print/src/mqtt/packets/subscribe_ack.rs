use common::serde::{DynamicSerializer, Serializer};

use crate::mqtt::Packet;

use super::QoS;

pub struct SubscribeAckPacket {
    pub packet_id: u16,
    pub return_codes: Vec<SubscribeReturnCode>,
}

#[derive(Clone, Copy)]
pub enum SubscribeReturnCode {
    Success(QoS),
    Failure,
}

impl SubscribeAckPacket {
    pub const PACKET_TYPE: u8 = 0x09;

    pub fn to_packet(&self) -> Packet {
        let mut ser = DynamicSerializer::new();
        ser.write_u16_be(self.packet_id);
        for return_code in &self.return_codes {
            match return_code {
                SubscribeReturnCode::Failure => ser.write_u8(0x80),
                SubscribeReturnCode::Success(qos) => ser.write_u8(qos.0),
            }
        }

        let data = ser.into_inner();
        Packet {
            packet_type: Self::PACKET_TYPE,
            flags: 0,
            remaining_length: data.len() as u32,
            remaining_bytes: data,
        }
    }
}
