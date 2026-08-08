use std::{
    io::ErrorKind,
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    ops::Deref,
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use anyhow::{Context, Result, bail};
use common::slice::format::RasterFormat;
use tracing::info;

use crate::{
    shared::{PrintInfo, Response, addr},
    v1::{
        self, RemotePrintV1,
        status::{FileTransferInfo, FileTransferStatus, FullStatusData},
    },
    v3::{self, RemotePrintV3, status::DiscoveryResponse},
};

#[derive(Default)]
pub struct RemotePrintManager {
    inner: Option<Arc<RemotePrintManagerInner>>,
}

pub struct RemotePrintManagerInner {
    pub v1: RemotePrintV1,
    pub v3: RemotePrintV3,

    pub udp: UdpSocket,
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
    pub fn init(
        &mut self,
        (udp, mqqt, http): (u16, u16, u16),
        timeout: Duration,
        print_completion: impl FnMut(&Client) + Send + Sync + 'static,
    ) -> Result<()> {
        assert!(self.inner.is_none());

        let udp = UdpSocket::bind(addr(udp)).context("Failed to bind UDP")?;
        udp.set_read_timeout(Some(timeout))?;
        udp.set_broadcast(true)?;

        let mut v1 = RemotePrintV1::uninitialized();
        v1.init((mqqt, http), print_completion)?;

        self.inner = Some(Arc::new(RemotePrintManagerInner {
            v1,
            v3: RemotePrintV3::default(),
            udp_port: udp.local_addr()?.port(),
            udp,
        }));
        Ok(())
    }

    pub fn shutdown(&mut self) {
        self.inner.take();
    }

    pub fn is_initialized(&self) -> bool {
        self.inner.is_some()
    }

    pub fn inner(&self) -> Option<Arc<RemotePrintManagerInner>> {
        self.inner.clone()
    }

    // not ideal allocating every frame but its whatever...
    pub fn clients(&self) -> Vec<Client> {
        let v1_clients = self.v1.clients();
        let v3_clients = self.v3.clients();

        (v1_clients.values().map(Client::from_v1))
            .chain(v3_clients.values().flat_map(Client::from_v3))
            .collect()
    }
}

impl RemotePrintManagerInner {
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
            ProtocolVersion::V3 => self.v3.remove_printer(mainboard),
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
            ProtocolVersion::V3 => self.v3.upload(mainboard, data, filename, format),
        }
    }

    pub fn print(&self, mainboard: &str, filename: &str) -> Result<()> {
        match self.protocol_version(mainboard).unwrap() {
            ProtocolVersion::V1 => self.v1.print(mainboard, filename),
            ProtocolVersion::V3 => self.v3.print(mainboard, filename),
        }
    }

    pub fn set_timeout(&self, timeout: Duration) -> Result<()> {
        self.udp.set_read_timeout(Some(timeout))?;
        Ok(())
    }

    fn on_response(&self, address: SocketAddr, received: &str) -> Result<()> {
        if let Ok(response) = serde_json::from_str::<Response<DiscoveryResponse>>(received) {
            self.v3.connect_printer(response)?;
        } else if let Ok(response) = serde_json::from_str::<Response<FullStatusData>>(received) {
            self.v1.connect_printer(&self.udp, response, address)?;
        } else {
            bail!("Received invalid response from printer.");
        }

        Ok(())
    }

    pub fn add_printer(&self, address: Ipv4Addr) -> Result<()> {
        info!("Attempting to connect to printer at {address}");
        let address = SocketAddr::new(address.into(), 3000);
        self.udp.send_to(b"M99999", address)?;

        let mut buffer = [0; 1024];
        let (len, _addr) = (self.udp)
            .recv_from(&mut buffer)
            .context("No response from printer.")?;

        let received = String::from_utf8_lossy(&buffer[..len]);
        self.on_response(address, &received)?;
        Ok(())
    }

    pub fn scan(&self, broadcast: Ipv4Addr) -> Result<()> {
        info!("Scanning for printers on {broadcast}");
        (self.udp).send_to(b"M99999", SocketAddr::new(broadcast.into(), 3000))?;

        let mut buffer = [0; 1024];
        loop {
            let (len, address) = match self.udp.recv_from(&mut buffer) {
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
    pub fn from_v1(client: &v1::mqtt_server::MqttClient) -> Self {
        let status = client.status.lock();
        Client {
            mainboard: client.attributes.mainboard_id.clone(),
            name: client.attributes.name.clone(),
            last_update: client.last_update.load(Ordering::Relaxed),
            print_info: status.print_info.clone(),
            transfer_info: status.file_transfer_info.clone(),
        }
    }

    pub fn from_v3(client: &v3::Client) -> Option<Self> {
        let (attributes, status) = (client.attributes.as_ref()?, client.status.as_ref()?);
        Some(Client {
            mainboard: attributes.mainboard_id.clone(),
            name: attributes.name.clone(),
            last_update: client.last_update,
            print_info: status.print_info.clone(),
            // TODO ↓
            transfer_info: FileTransferInfo {
                status: FileTransferStatus::None,
                download_offset: 0,
                check_offset: 0,
                file_total_size: 0,
                filename: "".into(),
            },
        })
    }
}

impl Deref for RemotePrintManager {
    type Target = Arc<RemotePrintManagerInner>;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}
