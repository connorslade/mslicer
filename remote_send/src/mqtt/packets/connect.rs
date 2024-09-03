use anyhow::Result;
use bitflags::bitflags;

use common::serde::Deserializer;

use crate::mqtt::misc::MqttDeserialize;

use super::Packet;

#[derive(Debug)]
pub struct ConnectPacket {
    pub protocol_name: String,
    pub protocol_level: u8,
    pub connect_flags: ConnectFlags,
    pub keep_alive: u16,

    pub client_id: String,
    pub will_topic: Option<String>,
    pub will_message: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

bitflags! {
    #[derive(Debug)]
    pub struct ConnectFlags: u8 {
        const USERNAME = 0b10000000;
        const PASSWORD = 0b01000000;
        const WILL_RETAIN = 0b00100000;
        const WILL_QOS = 0b00011000;
        const WILL_FLAG = 0b00000100;
        const CLEAN_SESSION = 0b00000010;
        const RESERVED = 0b00000001;
    }
}

impl ConnectPacket {
    pub const PACKET_TYPE: u8 = 0x01;

    pub fn from_packet(packet: &Packet) -> Result<Self> {
        let mut des = Deserializer::new(&packet.remaining_bytes);

        let protocol_name = des.read_string().into_owned();
        let protocol_level = des.read_u8();
        let connect_flags = ConnectFlags::from_bits(des.read_u8()).unwrap();
        let keep_alive = des.read_u16_be();

        let client_id = des.read_string().into_owned();
        let will_topic = connect_flags
            .contains(ConnectFlags::WILL_FLAG)
            .then(|| des.read_string().into_owned());
        let will_message = will_topic
            .as_ref()
            .and_then(|_| des.read_string().into_owned().into());
        let username = connect_flags
            .contains(ConnectFlags::USERNAME)
            .then(|| des.read_string().into_owned());
        let password = connect_flags
            .contains(ConnectFlags::PASSWORD)
            .then(|| des.read_string().into_owned());

        Ok(Self {
            protocol_name,
            protocol_level,
            connect_flags,
            keep_alive,
            client_id,
            will_topic,
            will_message,
            username,
            password,
        })
    }
}
