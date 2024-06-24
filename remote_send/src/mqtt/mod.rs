use std::{
    borrow::Cow,
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};

use anyhow::Result;
use common::serde::Deserializer;
use misc::next_id;
use packets::{
    connect::ConnectPacket,
    connect_ack::{ConnectAckFlags, ConnectAckPacket, ConnectReturnCode},
    subscribe::SubscribePacket,
};
use parking_lot::{lock_api::MutexGuard, MappedMutexGuard, Mutex};

mod misc;
pub mod packets;

pub struct MqttServer {
    clients: Mutex<HashMap<u64, MqttClient>>,
}

#[derive(Debug)]
pub struct MqttClient {
    stream: TcpStream,
    subscriptions: Vec<String>,
}

impl MqttServer {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            clients: Mutex::new(HashMap::new()),
        })
    }

    pub fn start_async(self: Arc<Self>) -> Result<()> {
        let socket = TcpListener::bind("0.0.0.0:1883")?;

        thread::spawn(move || {
            for stream in socket.incoming() {
                let stream = stream.unwrap();
                let client_id = next_id();
                self.clients
                    .lock()
                    .insert(client_id, MqttClient::new(stream.try_clone().unwrap()));

                println!("Connection established: {:?}", stream);

                let this_self = self.clone();
                thread::spawn(move || {
                    if let Err(e) = handle_client(this_self, client_id, stream) {
                        eprintln!("Error handling client: {:?}", e);
                    }
                });
            }
        });

        Ok(())
    }

    fn get_client_mut(&self, client_id: u64) -> MappedMutexGuard<MqttClient> {
        MutexGuard::map(self.clients.lock(), |x| x.get_mut(&client_id).unwrap())
    }

    fn remove_client(&self, client_id: u64) {
        self.clients.lock().remove(&client_id);
    }
}

impl MqttClient {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            subscriptions: Vec::new(),
        }
    }
}

fn handle_client(server: Arc<MqttServer>, client_id: u64, mut stream: TcpStream) -> Result<()> {
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
                server
                    .get_client_mut(client_id)
                    .subscriptions
                    .extend(packet.filters.into_iter().map(|x| x.0));
                dbg!(&server.get_client_mut(client_id));
            }
            0x0E => {
                println!("Client disconnect: {client_id}");
                server.remove_client(client_id);
                break;
            }
            ty => eprintln!("Unsupported packet type: 0x{ty:x}"),
        }
    }

    Ok(())
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
