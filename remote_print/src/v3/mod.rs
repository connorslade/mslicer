use std::collections::HashMap;

use anyhow::Result;
use parking_lot::Mutex;
use tracing::{info, warn};

use crate::{shared::Response, v3::status::DiscoveryResponse};

pub mod status;

#[derive(Default)]
pub struct RemotePrintV3 {
    clients: Mutex<HashMap<String, Client>>,
}

pub struct Client {}

impl RemotePrintV3 {
    pub(crate) fn connect_printer(&self, response: Response<DiscoveryResponse>) -> Result<()> {
        let machine_name = &response.data.machine_name;
        let mainboard_id = &response.data.mainboard_id;
        info!("Got status from `{machine_name}`",);

        if self.clients.lock().contains_key(mainboard_id) {
            warn!("Printer `{mainboard_id}` already connected.",);
            return Ok(());
        }

        let ip = &response.data.mainboard_ip;
        let (socket, response) = tungstenite::connect(format!("ws://{ip}:3030/websocket"))?;

        unimplemented!();

        Ok(())
    }
}
