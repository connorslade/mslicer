use std::{
    collections::HashMap,
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};

use anyhow::Result;
use misc::next_id;
use packets::{
    connect::ConnectPacket,
    connect_ack::{ConnectAckFlags, ConnectAckPacket, ConnectReturnCode},
    publish::{PublishFlags, PublishPacket},
    publish_ack::PublishAckPacket,
    subscribe::SubscribePacket,
    subscribe_ack::{SubscribeAckPacket, SubscribeReturnCode},
    Packet,
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
                let packet = ConnectPacket::from_packet(&packet)?;
                println!("Connect packet: {:?}", packet);

                ConnectAckPacket {
                    flags: ConnectAckFlags::empty(),
                    return_code: ConnectReturnCode::Accepted,
                }
                .to_packet()
                .write(&mut stream)?;
            }
            SubscribePacket::PACKET_TYPE => {
                let packet = SubscribePacket::from_packet(&packet)?;
                println!("Subscribe packet: {:?}", packet);

                let return_codes = packet
                    .filters
                    .iter()
                    .map(|(_topic, qos)| SubscribeReturnCode::Success(*qos))
                    .collect();

                server
                    .get_client_mut(client_id)
                    .subscriptions
                    .extend(packet.filters.into_iter().map(|x| x.0));
                dbg!(&server.get_client_mut(client_id));

                SubscribeAckPacket {
                    packet_id: packet.packet_id,
                    return_codes,
                }
                .to_packet()
                .write(&mut stream)?;
            }
            PublishPacket::PACKET_TYPE => {
                let packet = PublishPacket::from_packet(&packet)?;
                println!("Publish packet: {:?}", packet);
                println!("{}", String::from_utf8_lossy(&packet.data));

                if packet.flags.contains(PublishFlags::QOS1) {
                    PublishAckPacket {
                        packet_id: packet.packet_id.unwrap(),
                    }
                    .to_packet()
                    .write(&mut stream)?;
                }
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
