use std::{collections::HashMap, sync::Arc, thread, time::Duration};

use anyhow::Result;
use clone_macro::clone;
use common::slice::format::RasterFormat;
use parking_lot::{Mutex, MutexGuard};
use rand::{RngExt, rng};
use serde_json::{Map, Value, json};
use tracing::{info, trace, warn};
use tungstenite::Error;

use crate::{
    shared::{Response, epoch},
    v3::status::{Attributes, Command, CommandData, DiscoveryResponse, Message, Status},
};

pub mod status;

#[derive(Default)]
pub struct RemotePrintV3 {
    clients: Arc<Mutex<HashMap<String, Client>>>,
}

#[derive(Default)]
pub struct Client {
    pub attributes: Option<Attributes>,
    pub status: Option<Status>,
    pub last_update: i64,
    pub pending_removal: bool,
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
        let (mut socket, response) = tungstenite::connect(format!("ws://{ip}:3030/websocket"))?;
        trace!("Websocket connected: {response:?}");

        {
            let message = serde_json::to_string(&Command {
                id: mainboard_id.clone(),
                data: CommandData {
                    cmd: 0,
                    data: json!({}),
                    request_id: hex::encode(rng().random::<[u8; 8]>()),
                    mainboard_id: mainboard_id.clone(),
                    time_stamp: epoch(),
                    from: 0,
                },
                topic: format!("sdcp/request/{mainboard_id}"),
            })
            .unwrap();
            socket
                .send(tungstenite::Message::Text(message.into()))
                .unwrap();
        }

        {
            let message = serde_json::to_string(&Command {
                id: mainboard_id.clone(),
                data: CommandData {
                    cmd: 1,
                    data: json!({}),
                    request_id: hex::encode(rng().random::<[u8; 8]>()),
                    mainboard_id: mainboard_id.clone(),
                    time_stamp: epoch(),
                    from: 0,
                },
                topic: format!("sdcp/request/{mainboard_id}"),
            })
            .unwrap();
            socket
                .send(tungstenite::Message::Text(message.into()))
                .unwrap();
        }

        trace!("Sent commands");

        thread::spawn(clone!([{ self.clients } as clients], move || {
            loop {
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

                // todo: does this block?
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
                        let client = clients.entry(mainboard_id.clone()).or_default();
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

    pub fn print(&self, _mainboard: &str, _filename: &str) -> Result<()> {
        Ok(())
    }
}
