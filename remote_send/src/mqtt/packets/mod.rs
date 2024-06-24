use std::io::{Read, Write};

use anyhow::Result;

pub mod connect;
pub mod connect_ack;
pub mod publish;
pub mod publish_ack;
pub mod subscribe;
pub mod subscribe_ack;

pub struct Packet {
    pub packet_type: u8,
    pub flags: u8,
    pub remaining_length: u32,
    pub remaining_bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct QoS(pub u8);

impl Packet {
    pub fn write<Stream: Write>(&self, stream: &mut Stream) -> Result<()> {
        let mut bytes = vec![self.packet_type << 4 | self.flags];
        let mut remaining_length = self.remaining_length;
        loop {
            let mut byte = (remaining_length % 128) as u8;
            remaining_length /= 128;
            if remaining_length > 0 {
                byte |= 0x80;
            }
            bytes.push(byte);
            if remaining_length == 0 {
                break;
            }
        }
        bytes.extend(self.remaining_bytes.iter());

        stream.write_all(&bytes)?;
        Ok(())
    }

    pub fn read<Stream: Read>(stream: &mut Stream) -> Result<Self> {
        let mut header = vec![0; 2];
        stream.read_exact(&mut header)?;

        let (packet_type, flags) = (header[0] >> 4, header[0] & 0xF);
        let mut multiplier = 1;
        let mut remaining_length = 0;
        let mut pos = 1;
        loop {
            if pos == header.len() {
                header.resize(header.len() + 1, 0);
                stream.read_exact(&mut header[pos..])?;
            }

            let byte = header[pos];
            remaining_length += (byte & 0x7F) as u32 * multiplier;
            multiplier *= 128;
            pos += 1;
            if byte & 0x80 == 0 {
                break;
            }
        }

        let mut remaining_bytes = vec![0; remaining_length as usize];
        stream.read_exact(&mut remaining_bytes)?;

        Ok(Self {
            packet_type,
            flags,
            remaining_length,
            remaining_bytes,
        })
    }
}
