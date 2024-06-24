use bitflags::bitflags;

use crate::mqtt::Packet;

pub struct ConnectAckPacket {
    pub flags: ConnectAckFlags,
    pub return_code: ConnectReturnCode,
}

bitflags! {
    pub struct ConnectAckFlags: u8 {
        const SESSION_PRESENT = 0b00000001;
    }
}

pub enum ConnectReturnCode {
    Accepted,
    Refused(ConnectRefusedReason),
}

pub enum ConnectRefusedReason {
    UnacceptableProtocolVersion,
    IdentifierRejected,
    ServerUnavailable,
    BadUsernameOrPassword,
    NotAuthorized,
}

impl ConnectAckPacket {
    const PACKET_TYPE: u8 = 0x02;

    pub fn to_packet(&self) -> Packet {
        let body = vec![self.flags.bits(), self.return_code.as_u8()];

        Packet {
            packet_type: Self::PACKET_TYPE,
            flags: 0,
            remaining_length: body.len() as u32,
            remaining_bytes: body,
        }
    }
}

impl ConnectReturnCode {
    fn as_u8(&self) -> u8 {
        match self {
            ConnectReturnCode::Accepted => 0,
            ConnectReturnCode::Refused(reason) => match reason {
                ConnectRefusedReason::UnacceptableProtocolVersion => 1,
                ConnectRefusedReason::IdentifierRejected => 2,
                ConnectRefusedReason::ServerUnavailable => 3,
                ConnectRefusedReason::BadUsernameOrPassword => 4,
                ConnectRefusedReason::NotAuthorized => 5,
            },
        }
    }
}
