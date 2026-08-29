use std::{
    collections::HashMap,
    io::ErrorKind,
    mem,
    net::{Ipv4Addr, SocketAddr, TcpStream},
    sync::{
        Arc,
        mpsc::{self, Sender},
    },
    thread,
    time::Duration,
};

use anyhow::{Context, Result, bail};
use clone_macro::clone;
use parking_lot::{Mutex, MutexGuard};
use tracing::{info, trace, warn};
use tungstenite::Error;
use ureq::unversioned::multipart::{Form, Part};
use uuid::Uuid;

use crate::{
    manager,
    shared::{FileTransferInfo, FileTransferStatus, Response, epoch},
    v3::{
        commands::{Cmd, send_command},
        status::{
            Attributes, DiscoveryResponse, ErrorMessage, Message, NoticeMessage, ResponseMessage,
            Status, UploadResponse,
        },
    },
};

pub mod commands;
pub mod status;

type PrintCompletion = Arc<Mutex<Box<dyn FnMut(&manager::Client) + Send>>>;

pub struct RemotePrintV3 {
    clients: Arc<Mutex<HashMap<String, Client>>>,
    print_completion: PrintCompletion,
}

pub struct Client {
    pub attributes: Option<Attributes>,
    pub status: Option<Status>,
    pub transfer_info: FileTransferInfo,

    pub ip: Ipv4Addr,
    pub last_update: i64,
    pub pending_removal: bool,
    pub sender: Sender<Cmd>,
    was_printing: bool,
}

impl RemotePrintV3 {
    pub(crate) fn new(
        print_completion: impl FnMut(&manager::Client) + Send + Sync + 'static,
    ) -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            print_completion: Arc::new(Mutex::new(Box::new(print_completion))),
        }
    }

    pub(crate) fn connect_printer(&self, response: Response<DiscoveryResponse>) -> Result<()> {
        let machine_name = &response.data.machine_name;
        let mainboard_id = response.data.mainboard_id.clone();
        info!("Got status from `{machine_name}`",);

        if self.clients.lock().contains_key(&mainboard_id) {
            warn!("Printer `{mainboard_id}` already connected.",);
            return Ok(());
        }

        let ip = response.data.mainboard_ip.parse::<Ipv4Addr>().unwrap();
        let addr = SocketAddr::new(ip.into(), 3030);
        let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5))?;

        // Needed for elegoo-homeassistant proxy support
        let url = if response.data.proxy {
            format!("ws://{ip}:3030/websocket?id={mainboard_id}")
        } else {
            format!("ws://{ip}:3030/websocket")
        };

        let (mut socket, rsp) = tungstenite::client(url, stream)?;

        // note that the socket must only be configured after the websocket
        // handshake as tungstenite assumes blocking sockets. this was not fun
        // to figure out :sob:
        socket
            .get_ref()
            .set_read_timeout(Some(Duration::from_secs(1)))?;
        trace!("Websocket connected: {rsp:?}");

        let (tx, rx) = mpsc::channel();
        tx.send(Cmd::RefreshStatus).unwrap();
        tx.send(Cmd::RefreshAttributes).unwrap();

        self.clients
            .lock()
            .insert(mainboard_id.clone(), Client::new(ip, tx));

        thread::spawn(clone!(
            [{ self.clients } as clients, { self.print_completion }
                as print_completion],
            move || {
                loop {
                    while let Ok(command) = rx.try_recv() {
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

                            // Ignore heartbeat msgs
                            if matches!(text, "ping" | "pong") {
                                continue;
                            }

                            if let Ok(message) = serde_json::from_str::<Message>(text) {
                                trace!("message: {message:?}");
                                let mut clients = clients.lock();
                                let client = clients.get_mut(&mainboard_id).unwrap();

                                if let Some(attributes) = message.attributes {
                                    trace!("Got attributes");
                                    client.attributes = Some(attributes);
                                }

                                if let Some(status) = message.status {
                                    trace!("Got status");
                                    let is_printing = status.print_info.status.is_printing();
                                    client.status = Some(status);

                                    if mem::replace(&mut client.was_printing, is_printing)
                                        && !is_printing
                                        && let Some(client) = manager::Client::from_v3(client)
                                    {
                                        print_completion.lock()(&client)
                                    }
                                }
                            } else if let Ok(response) =
                                serde_json::from_str::<ResponseMessage>(text)
                            {
                                let (cmd, ack) = (response.data.cmd, response.data.data.ack);
                                if ack != 0 {
                                    warn!(
                                        "Printer `{mainboard_id}` rejected command {cmd} with {ack}"
                                    );
                                } else {
                                    trace!("Printer `{mainboard_id}` acknowledged command {cmd}");
                                }
                            } else if let Ok(error) = serde_json::from_str::<ErrorMessage>(text) {
                                warn!(
                                    "Printer `{mainboard_id}` reported error {:?}",
                                    error.data.data.error_code
                                );
                            } else if let Ok(notice) = serde_json::from_str::<NoticeMessage>(text) {
                                info!(
                                    "Printer `{mainboard_id}` notice (type {}): {:?}",
                                    notice.data.data.notice_type, notice.data.data.message
                                );
                            } else {
                                warn!("Failed to deserialize message: {text:?}");
                            }

                            if let Some(client) = clients.lock().get_mut(&mainboard_id) {
                                client.last_update = epoch();
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
                }
            }
        ));

        Ok(())
    }

    pub fn clients(&self) -> MutexGuard<'_, HashMap<String, Client>> {
        self.clients.lock()
    }

    pub fn remove_printer(&self, mainboard: &str) -> Result<()> {
        self.clients.lock().remove(mainboard);
        Ok(())
    }

    pub fn upload(&self, mainboard: &str, data: Arc<Vec<u8>>, filename: String) -> Result<()> {
        let mut clients = self.clients.lock();
        let client = clients.get_mut(mainboard).unwrap();
        client.transfer_info = FileTransferInfo {
            status: FileTransferStatus::None,
            download_offset: 0,
            check_offset: 0,
            file_total_size: data.len() as u32,
            filename: filename.clone(),
        };
        let ip = client.ip;
        drop(clients);

        let result = self.upload_chunks(mainboard, ip, &data, &filename);

        let mut clients = self.clients.lock();
        if let Some(client) = clients.get_mut(mainboard) {
            client.transfer_info.status =
                [FileTransferStatus::Error, FileTransferStatus::Done][result.is_ok() as usize];
        }

        result
    }

    fn upload_chunks(
        &self,
        mainboard: &str,
        ip: Ipv4Addr,
        data: &[u8],
        filename: &str,
    ) -> Result<()> {
        const CHUNK_SIZE: usize = 1024 * 1024;

        let md5 = format!("{:x}", md5::compute(data));
        let uuid = Uuid::new_v4().to_string();
        let total_size = data.len();
        let total_size_str = total_size.to_string();
        let url = format!("http://{ip}:3030/uploadFile/upload");

        let mut offset = 0;
        while offset < total_size {
            let end = (offset + CHUNK_SIZE).min(total_size);
            let chunk = &data[offset..end];
            let offset_str = offset.to_string();

            let file = Part::bytes(chunk)
                .file_name(filename)
                .mime_str("application/octet-stream")?;
            let form = Form::new()
                .text("S-File-MD5", &md5)
                .text("Check", "1")
                .text("Offset", &offset_str)
                .text("Uuid", &uuid)
                .text("TotalSize", &total_size_str)
                .part("File", file);

            let mut response = ureq::post(&url).send(form)?;
            let body = response.body_mut().read_to_string()?;
            let upload = serde_json::from_str::<UploadResponse>(&body)?;
            if !upload.success {
                let messages = &upload.messages;
                bail!("Printer rejected file chunk at offset {offset}: {messages:?}");
            }

            offset = end;
            if let Some(client) = self.clients.lock().get_mut(mainboard) {
                client.transfer_info.download_offset = offset as u32;
                client.transfer_info.check_offset = offset as u32;
            }
        }

        Ok(())
    }

    pub fn print(&self, mainboard: &str, filename: &str) -> Result<()> {
        let cmd = Cmd::StartPrinting {
            filename: filename.to_owned(),
            start_layer: 0,
        };
        self.send_cmd(mainboard, cmd)
    }

    pub fn pause_print(&self, mainboard: &str) -> Result<()> {
        self.send_cmd(mainboard, Cmd::PausePrinting)
    }

    pub fn resume_print(&self, mainboard: &str) -> Result<()> {
        self.send_cmd(mainboard, Cmd::ResumePrinting)
    }

    pub fn stop_print(&self, mainboard: &str) -> Result<()> {
        self.send_cmd(mainboard, Cmd::StopPrinting)
    }

    fn send_cmd(&self, mainboard: &str, cmd: Cmd) -> Result<()> {
        let clients = self.clients();
        let client = clients
            .get(mainboard)
            .with_context(|| format!("Printer `{mainboard}` is not connected."))?;
        client.sender.send(cmd).unwrap();
        Ok(())
    }
}

impl Client {
    fn new(ip: Ipv4Addr, sender: Sender<Cmd>) -> Self {
        Self {
            attributes: None,
            status: None,
            transfer_info: FileTransferInfo {
                status: FileTransferStatus::None,
                download_offset: 0,
                check_offset: 0,
                file_total_size: 0,
                filename: "".into(),
            },

            ip,
            last_update: 0,
            pending_removal: false,
            sender,
            was_printing: false,
        }
    }
}
