use std::{
    borrow::Cow,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use anyhow::Result;
use common::serde::Deserializer;
use packets::{
    connect::ConnectPacket,
    connect_ack::{ConnectAckFlags, ConnectAckPacket, ConnectReturnCode},
    subscribe::SubscribePacket,
};

pub mod packets;

pub fn start() -> Result<()> {
    let socket = TcpListener::bind("0.0.0.0:1883")?;

    for stream in socket.incoming() {
        let stream = stream?;
        println!("Connection established: {:?}", stream);
        thread::spawn(|| {
            if let Err(e) = handle_client(stream) {
                eprintln!("Error handling client: {:?}", e);
            }
        });
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream) -> Result<()> {
    loop {
        let packet = Packet::read(&mut stream)?;

        match packet.packet_type {
            ConnectPacket::PACKET_TYPE => {
                let packet = ConnectPacket::from_bytes(&packet.remaining_bytes)?;
                println!("Connect packet: {:?}", packet);

                ConnectAckPacket {
                    flags: ConnectAckFlags::empty(),
                    return_code: ConnectReturnCode::Accepted,
                }
                .to_packet()
                .write(&mut stream)?;
            }
            SubscribePacket::PACKET_TYPE => {
                let packet = SubscribePacket::from_bytes(&packet.remaining_bytes)?;
                println!("Subscribe packet: {:?}", packet);
            }
            ty => eprintln!("Unsupported packet type: 0x{ty:x}"),
        }
    }

}

pub struct Packet {
    packet_type: u8,
    flags: u8,
    remaining_length: u32,
    remaining_bytes: Vec<u8>,
}

impl Packet {
    fn write<Stream: Write>(&self, stream: &mut Stream) -> Result<()> {
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

    fn read<Stream: Read>(stream: &mut Stream) -> Result<Self> {
        let mut header = [0; 2];
        stream.read_exact(&mut header)?;

        let (packet_type, flags) = (header[0] >> 4, header[0] & 0xF);
        let mut multiplier = 1;
        let mut remaining_length = 0;
        let mut pos = 1;
        loop {
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

trait MqttDeserialize<'a> {
    fn read_string(&mut self) -> Cow<'a, str>;
}

impl<'a> MqttDeserialize<'a> for Deserializer<'a> {
    fn read_string(&mut self) -> Cow<'a, str> {
        let len = self.read_u16();
        let buf = self.read_bytes(len as usize);
        String::from_utf8_lossy(buf)
    }
}
