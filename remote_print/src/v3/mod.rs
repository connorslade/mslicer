use std::{
    collections::HashMap,
    io::ErrorKind,
    net::{SocketAddr, TcpStream},
    sync::{
        Arc,
        mpsc::{self, Sender},
    },
    thread,
    time::Duration,
};

use anyhow::Result;
use clone_macro::clone;
use common::slice::format::RasterFormat;
use parking_lot::{Mutex, MutexGuard};
use tracing::{info, trace, warn};
use tungstenite::Error;

use crate::{
    shared::{Response, epoch},
    v3::{
        commands::{Cmd, send_command},
        status::{Attributes, DiscoveryResponse, Message, Status},
    },
};

pub mod commands;
pub mod status;

#[derive(Default)]
pub struct RemotePrintV3 {
    clients: Arc<Mutex<HashMap<String, Client>>>,
}

pub struct Client {
    pub attributes: Option<Attributes>,
    pub status: Option<Status>,
    pub last_update: i64,
    pub pending_removal: bool,
    pub sender: Sender<Cmd>,
}

impl RemotePrintV3 {
    pub(crate) fn connect_printer(&self, response: Response<DiscoveryResponse>) -> Result<()> {
        let machine_name = &response.data.machine_name;
        let mainboard_id = response.data.mainboard_id.clone();
        info!("Got status from `{machine_name}`",);

        if self.clients.lock().contains_key(&mainboard_id) {
            warn!("Printer `{mainboard_id}` already connected.",);
            return Ok(());
        }

        let ip = &response.data.mainboard_ip;
        let stream = TcpStream::connect(SocketAddr::new(ip.parse().unwrap(), 3000))?;
        stream.set_nonblocking(true)?;

        let (mut socket, rsp) = tungstenite::client(format!("ws://{ip}:3030/websocket"), stream)?;
        trace!("Websocket connected: {rsp:?}");

        let (tx, rx) = mpsc::channel();
        tx.send(Cmd::RefreshStatus).unwrap();
        tx.send(Cmd::RefreshAttributes).unwrap();

        self.clients
            .lock()
            .insert(mainboard_id.clone(), Client::new(tx));

        thread::spawn(clone!([{ self.clients } as clients], move || {
            loop {
                while let Ok(command) = rx.recv() {
                    send_command(&mut socket, &mainboard_id, command);
                }

                {
                    let mut clients = clients.lock();
                    if clients
                        .get(&mainboard_id)
                        .map(|x| x.pending_removal)
                        .unwrap_or_default()
                    {
                        clients.remove(&mainboard_id);
                        socket.close(None).unwrap();
                    }
                }

                match socket.read() {
                    Ok(message) => {
                        let text = match message.to_text() {
                            Ok(x) => x,
                            Err(e) => {
                                warn!("Failed to convert message to text: {e:?}");
                                continue;
                            }
                        };

                        trace!("text: {text:?}");
                        let message = match serde_json::from_str::<Message>(text) {
                            Ok(x) => x,
                            Err(e) => {
                                warn!("Failed to deserialize message: {e:?}");
                                continue;
                            }
                        };

                        trace!("message: {message:?}");
                        let mut clients = clients.lock();
                        let client = clients.get_mut(&mainboard_id).unwrap();
                        client.last_update = epoch();

                        if let Some(attributes) = message.attributes {
                            trace!("Got attributes");
                            client.attributes = Some(attributes);
                        }

                        if let Some(status) = message.status {
                            trace!("Got status");
                            client.status = Some(status);
                        }
                    }
                    Err(Error::Io(e))
                        if matches!(e.kind(), ErrorKind::TimedOut | ErrorKind::WouldBlock) =>
                    {
                        continue;
                    }
                    Err(Error::ConnectionClosed) => {
                        trace!("Connection to `{mainboard_id}` closed.");
                        break;
                    }
                    Err(e) => warn!("Socket error for `{mainboard_id}`: {e:?}"),
                }

                thread::sleep(Duration::from_secs(1));
            }
        }));

        Ok(())
    }

    pub fn clients(&self) -> MutexGuard<'_, HashMap<String, Client>> {
        self.clients.lock()
    }

    pub fn remove_printer(&self, mainboard: &str) -> Result<()> {
        self.clients.lock().remove(mainboard);
        Ok(())
    }

    pub fn upload(
        &self,
        _mainboard: &str,
        _data: Arc<Vec<u8>>,
        _filename: String,
        _format: RasterFormat,
    ) -> Result<()> {
        Ok(())
    }

    pub fn print(&self, mainboard: &str, filename: &str) -> Result<()> {
        let cmd = Cmd::StartPrinting {
            filename: filename.to_owned(),
            start_layer: 0,
        };

        let clients = self.clients();
        let client = clients.get(mainboard).unwrap();
        client.sender.send(cmd).unwrap();
        Ok(())
    }
}

impl Client {
    fn new(sender: Sender<Cmd>) -> Self {
        Self {
            attributes: None,
            status: None,
            last_update: 0,
            pending_removal: false,
            sender,
        }
    }
}
