use std::{
    collections::HashMap,
    io::ErrorKind,
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
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
use soon::Soon;
use tracing::{info, trace, warn};

mod misc;
pub mod packets;

pub struct MqttServer<H: MqttHandler> {
    shutdown: AtomicBool,
    listener: Mutex<Option<TcpListener>>,
    clients: Mutex<HashMap<u64, TcpStream>>,
    handler: Soon<H>,
}

pub trait MqttHandler
where
    Self: Sized,
{
    fn init(&self, server: &Arc<MqttServer<Self>>);
    fn on_connect(&self, client_id: u64, packet: ConnectPacket) -> Result<ConnectAckPacket>;
    fn on_subscribe(&self, client_id: u64, packet: SubscribePacket) -> Result<SubscribeAckPacket>;
    fn on_publish(&self, client_id: u64, packet: PublishPacket) -> Result<()>;
    fn on_publish_ack(&self, client_id: u64, packet: PublishAckPacket) -> Result<()>;
    fn on_disconnect(&self, client_id: u64) -> Result<()>;
}

impl<H: MqttHandler + Send + Sync + 'static> MqttServer<H> {
    pub fn new(handler: H) -> Arc<Self> {
        let this = Arc::new(Self {
            shutdown: AtomicBool::new(false),
            listener: Mutex::new(None),
            clients: Mutex::new(HashMap::new()),
            handler: Soon::empty(),
        });

        handler.init(&this);
        this.handler.replace(handler);

        this
    }

    pub fn start_async(self: Arc<Self>, socket: TcpListener) -> Result<()> {
        *self.listener.lock() = Some(socket.try_clone()?);

        thread::spawn(move || {
            for stream in socket.incoming() {
                if self.shutdown.load(Ordering::Relaxed) {
                    break;
                }

                let stream = match stream {
                    Ok(stream) => stream,
                    Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                    Err(e) => {
                        warn!("Error accepting connection: {:?}", e);
                        continue;
                    }
                };

                let client_id = next_id();
                self.clients
                    .lock()
                    .insert(client_id, stream.try_clone().unwrap());

                info!("Connection established: {:?}", stream);

                let this_self = self.clone();
                thread::spawn(move || {
                    if let Err(e) = handle_client(this_self, client_id, stream) {
                        warn!("Error handling client: {:?}", e);
                    }
                });
            }

            trace!("Stopping MQTT event loop");
        });

        Ok(())
    }

    pub fn shutdown(self: Arc<Self>) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(listener) = self.listener.lock().take() {
            let port = listener.local_addr().unwrap().port();
            let socket_address = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), port);
            let _ = TcpStream::connect(socket_address).unwrap();
        }
    }

    pub fn send_packet(&self, client_id: u64, packet: Packet) -> Result<()> {
        let mut stream = self.get_client_mut(client_id);
        packet.write(&mut *stream)?;
        Ok(())
    }

    fn get_client_mut(&self, client_id: u64) -> MappedMutexGuard<'_, TcpStream> {
        MutexGuard::map(self.clients.lock(), |x| x.get_mut(&client_id).unwrap())
    }

    fn remove_client(&self, client_id: u64) {
        self.clients.lock().remove(&client_id);
    }
}

fn handle_client<H>(server: Arc<MqttServer<H>>, client_id: u64, mut stream: TcpStream) -> Result<()>
where
    H: MqttHandler + Send + Sync + 'static,
{
    loop {
        let packet = Packet::read(&mut stream)?;

        match packet.packet_type {
            ConnectPacket::PACKET_TYPE => {
                let packet = ConnectPacket::from_packet(&packet)?;

                server
                    .handler
                    .on_connect(client_id, packet)?
                    .to_packet()
                    .write(&mut stream)?;
            }
            SubscribePacket::PACKET_TYPE => {
                let packet = SubscribePacket::from_packet(&packet)?;

                server
                    .handler
                    .on_subscribe(client_id, packet)?
                    .to_packet()
                    .write(&mut stream)?;
            }
            PublishPacket::PACKET_TYPE => {
                let packet = PublishPacket::from_packet(&packet)?;
                let packet_id = packet
                    .flags
                    .contains(PublishFlags::QOS1)
                    .then(|| packet.packet_id.unwrap());

                server.handler.on_publish(client_id, packet)?;

                if let Some(packet_id) = packet_id {
                    PublishAckPacket { packet_id }
                        .to_packet()
                        .write(&mut stream)?;
                }
            }
            PublishAckPacket::PACKET_TYPE => {
                let packet = PublishAckPacket::from_packet(&packet)?;
                server.handler.on_publish_ack(client_id, packet)?;
            }
            0x0E => {
                server.handler.on_disconnect(client_id)?;
                server.remove_client(client_id);
                break;
            }
            ty => warn!("Unsupported packet type: 0x{ty:x}"),
        }
    }

    Ok(())
}
