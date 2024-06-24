use std::{
    collections::HashMap,
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicU16, Ordering},
        Arc,
    },
    thread,
};

use anyhow::Result;
use misc::next_id;
use packets::{
    connect::ConnectPacket,
    connect_ack::ConnectAckPacket,
    publish::{PublishFlags, PublishPacket},
    publish_ack::PublishAckPacket,
    subscribe::SubscribePacket,
    subscribe_ack::SubscribeAckPacket,
    Packet,
};
use parking_lot::{lock_api::MutexGuard, MappedMutexGuard, Mutex};

mod misc;
pub mod packets;

pub struct MqttServer<H: MqttHandler> {
    listeners: Mutex<Option<TcpListener>>,
    clients: Mutex<HashMap<u64, TcpStream>>,
    handler: H,
}

pub struct HandlerCtx<Handler: MqttHandler> {
    pub server: Arc<MqttServer<Handler>>,
    pub client_id: u64,
    next_packet_id: AtomicU16,
}

pub trait MqttHandler
where
    Self: Sized,
{
    fn on_connect(&self, ctx: &HandlerCtx<Self>, packet: ConnectPacket)
        -> Result<ConnectAckPacket>;
    fn on_subscribe(
        &self,
        ctx: &HandlerCtx<Self>,
        packet: SubscribePacket,
    ) -> Result<SubscribeAckPacket>;
    fn on_publish(&self, ctx: &HandlerCtx<Self>, packet: PublishPacket) -> Result<()>;
}

impl<H: MqttHandler + Send + Sync + 'static> MqttServer<H> {
    pub fn new(handler: H) -> Arc<Self> {
        Arc::new(Self {
            listeners: Mutex::new(None),
            clients: Mutex::new(HashMap::new()),
            handler,
        })
    }

    pub fn start_async(self: Arc<Self>) -> Result<()> {
        let socket = TcpListener::bind("0.0.0.0:1883")?;
        *self.listeners.lock() = Some(socket.try_clone()?);

        thread::spawn(move || {
            for stream in socket.incoming() {
                let Ok(stream) = stream else { continue };
                let client_id = next_id();
                self.clients
                    .lock()
                    .insert(client_id, stream.try_clone().unwrap());

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

    pub fn send_packet(&self, client_id: u64, packet: Packet) -> Result<()> {
        let mut stream = self.get_client_mut(client_id);
        packet.write(&mut *stream)?;
        Ok(())
    }

    fn get_client_mut(&self, client_id: u64) -> MappedMutexGuard<TcpStream> {
        MutexGuard::map(self.clients.lock(), |x| x.get_mut(&client_id).unwrap())
    }

    fn remove_client(&self, client_id: u64) {
        self.clients.lock().remove(&client_id);
    }
}

impl<H: MqttHandler> HandlerCtx<H> {
    pub fn next_packet_id(&self) -> u16 {
        self.next_packet_id.fetch_add(1, Ordering::Relaxed)
    }
}

fn handle_client<H>(server: Arc<MqttServer<H>>, client_id: u64, mut stream: TcpStream) -> Result<()>
where
    H: MqttHandler + Send + Sync + 'static,
{
    let ctx = HandlerCtx {
        server: server.clone(),
        client_id,
        next_packet_id: AtomicU16::new(0),
    };

    loop {
        let packet = Packet::read(&mut stream)?;

        match packet.packet_type {
            ConnectPacket::PACKET_TYPE => {
                let packet = ConnectPacket::from_packet(&packet)?;

                server
                    .handler
                    .on_connect(&ctx, packet)?
                    .to_packet()
                    .write(&mut stream)?;
            }
            SubscribePacket::PACKET_TYPE => {
                let packet = SubscribePacket::from_packet(&packet)?;

                server
                    .handler
                    .on_subscribe(&ctx, packet)?
                    .to_packet()
                    .write(&mut stream)?;
            }
            PublishPacket::PACKET_TYPE => {
                let packet = PublishPacket::from_packet(&packet)?;
                let packet_id = packet
                    .flags
                    .contains(PublishFlags::QOS1)
                    .then(|| packet.packet_id.unwrap());

                server.handler.on_publish(&ctx, packet)?;

                if let Some(packet_id) = packet_id {
                    PublishAckPacket { packet_id }
                        .to_packet()
                        .write(&mut stream)?;
                }
            }
            PublishAckPacket::PACKET_TYPE => {
                let packet = PublishAckPacket::from_packet(&packet)?;
                println!("Received publish ack {{ packet_id: {} }}", packet.packet_id);
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
