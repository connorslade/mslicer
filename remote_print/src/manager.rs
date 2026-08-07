use std::{
    io::ErrorKind,
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use anyhow::{Context, Result, bail};
use common::slice::format::RasterFormat;
use tracing::info;

use crate::{
    shared::{PrintInfo, Response, addr},
    v1::{
        RemotePrintV1,
        mqtt_server::MqttClient,
        status::{FileTransferInfo, FullStatusData},
    },
    v3::{RemotePrintV3, status::DiscoveryResponse},
};

pub struct RemotePrintManager {
    pub v1: RemotePrintV1,
    pub v3: RemotePrintV3,

    pub udp: Option<UdpSocket>,
    pub udp_port: u16,
}

pub enum ProtocolVersion {
    V1,
    V3,
}

pub struct Client {
    pub mainboard: String,
    pub name: String,
    pub last_update: i64,

    pub print_info: PrintInfo,
    pub transfer_info: FileTransferInfo,
}

impl RemotePrintManager {
    pub fn new() -> Self {
        Self {
            v1: RemotePrintV1::uninitialized(),
            v3: RemotePrintV3::default(),

            udp: None,
            udp_port: 0,
        }
    }

    pub fn init(
        &mut self,
        (udp, mqqt, http): (u16, u16, u16),
        timeout: Duration,
        print_completion: impl FnMut(&Client) + Send + Sync + 'static,
    ) -> Result<()> {
        let udp = UdpSocket::bind(addr(udp)).context("Failed to bind UDP")?;
        udp.set_read_timeout(Some(timeout))?;
        udp.set_broadcast(true)?;
        self.udp_port = udp.local_addr()?.port();
        self.udp = Some(udp);

        self.v1.init((mqqt, http), print_completion)?;
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.udp.is_some()
    }

    pub fn shutdown(&mut self) {
        self.v1.shutdown();
        self.udp.take();
        self.udp_port = 0;
    }

    pub fn udp(&self) -> &UdpSocket {
        self.udp.as_ref().unwrap()
    }

    // not ideal allocating every frame but its whatever...
    pub fn clients(&self) -> Vec<Client> {
        (self.v1.clients().iter())
            .map(|(_, c)| Client::from_v1(c))
            .collect()
    }
}

impl RemotePrintManager {
    pub fn protocol_version(&self, mainboard: &str) -> Option<ProtocolVersion> {
        if self.v1.clients().contains_key(mainboard) {
            Some(ProtocolVersion::V1)
        } else {
            None
        }
    }

    pub fn remove_printer(&self, mainboard: &str) -> Result<()> {
        match self.protocol_version(mainboard).unwrap() {
            ProtocolVersion::V1 => self.v1.remove_printer(mainboard),
            ProtocolVersion::V3 => todo!(),
        }
    }

    pub fn upload(
        &self,
        mainboard: &str,
        data: Arc<Vec<u8>>,
        filename: String,
        format: RasterFormat,
    ) -> Result<()> {
        match self.protocol_version(mainboard).unwrap() {
            ProtocolVersion::V1 => self.v1.upload(mainboard, data, filename, format),
            ProtocolVersion::V3 => todo!(),
        }
    }

    pub fn print(&self, mainboard: &str, filename: &str) -> Result<()> {
        match self.protocol_version(mainboard).unwrap() {
            ProtocolVersion::V1 => self.v1.print(mainboard, filename),
            ProtocolVersion::V3 => todo!(),
        }
    }
}

impl RemotePrintManager {
    pub fn set_timeout(&self, timeout: Duration) -> Result<()> {
        self.udp().set_read_timeout(Some(timeout))?;
        Ok(())
    }

    fn on_response(&self, address: SocketAddr, received: &str) -> Result<()> {
        if let Ok(response) = serde_json::from_str::<Response<DiscoveryResponse>>(&received) {
            self.v3.connect_printer(response)?;
        } else if let Ok(response) = serde_json::from_str::<Response<FullStatusData>>(&received) {
            self.v1.connect_printer(self.udp(), response, address)?;
        } else {
            bail!("Received invalid response from printer.");
        }

        Ok(())
    }

    pub fn add_printer(&self, address: Ipv4Addr) -> Result<()> {
        info!("Attempting to connect to printer at {address}");
        let address = SocketAddr::new(address.into(), 3000);
        self.udp().send_to(b"M99999", address)?;

        let mut buffer = [0; 1024];
        let (len, _addr) = (self.udp())
            .recv_from(&mut buffer)
            .context("No response from printer.")?;

        let received = String::from_utf8_lossy(&buffer[..len]);
        self.on_response(address, &received)?;
        Ok(())
    }

    pub fn scan(&self, broadcast: Ipv4Addr) -> Result<()> {
        info!("Scanning for printers on {broadcast}");
        (self.udp()).send_to(b"M99999", SocketAddr::new(broadcast.into(), 3000))?;

        let mut buffer = [0; 1024];
        loop {
            let (len, address) = match self.udp().recv_from(&mut buffer) {
                Ok(data) => data,
                Err(e) if matches!(e.kind(), ErrorKind::TimedOut | ErrorKind::WouldBlock) => break,
                Err(_) => continue,
            };

            let received = String::from_utf8_lossy(&buffer[..len]);
            self.on_response(address, &received)?;
        }

        Ok(())
    }
}

impl Client {
    pub fn from_v1(client: &MqttClient) -> Self {
        let status = client.status.lock();
        Client {
            mainboard: client.attributes.mainboard_id.clone(),
            name: client.attributes.name.clone(),
            last_update: client.last_update.load(Ordering::Relaxed),
            print_info: status.print_info.clone(),
            transfer_info: status.file_transfer_info.clone(),
        }
    }
}
