use std::{
    collections::HashMap,
    io::ErrorKind,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, UdpSocket},
    ops::Deref,
    sync::{Arc, Mutex, atomic::Ordering},
    time::Duration,
};

use anyhow::{Context, Result};
use common::{misc::random_string, slice::format::RasterFormat};
use parking_lot::{MappedRwLockReadGuard, RwLockReadGuard};
use tracing::{info, warn};

use crate::{
    mqtt::MqttServer,
    v1::{
        commands::{DisconnectCommand, StartPrinting, UploadFile},
        http_server::HttpServer,
        misc::Response,
        mqtt_server::{Mqtt, MqttClient},
        status::{FullStatusData, PrintInfo, PrintInfoStatus},
    },
};

pub mod commands;
pub mod http_server;
pub mod misc;
pub mod mqtt_server;
pub mod status;

pub struct RemotePrintV1 {
    services: Option<Arc<Services>>,
}

pub struct Services {
    pub mqtt: Mqtt,
    pub http: HttpServer,
    pub udp: UdpSocket,

    pub mqtt_port: u16,
    pub http_port: u16,
    pub udp_port: u16,
}

impl RemotePrintV1 {
    pub fn uninitialized() -> Self {
        Self { services: None }
    }

    pub fn services(&self) -> &Arc<Services> {
        self.services
            .as_ref()
            .expect("RemotePrintV1 not initialized.")
    }

    pub fn ports(&self) -> (u16, u16, u16) {
        let services = &self.services();
        (services.mqtt_port, services.http_port, services.udp_port)
    }

    pub fn init(
        &mut self,
        (p_mqqt, p_udp, p_http): (u16, u16, u16),
        timeout: Duration,
        print_completion: impl FnMut(&MqttClient, &PrintInfo) + Send + Sync + 'static,
    ) -> Result<()> {
        assert!(self.services.is_none());

        let print_completion = Mutex::new(print_completion);
        let addr = |port| SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);

        let mqtt_listener = TcpListener::bind(addr(p_mqqt)).context("Failed to bind MQTT")?;
        let mqtt_port = mqtt_listener.local_addr()?.port();
        let mqtt = Mqtt::new_callback(move |client| {
            let print_info = &client.status.lock().print_info;
            let is_printing = print_info.status.is_printing();
            let was_printing = client.was_printing.swap(is_printing, Ordering::Relaxed);
            (was_printing && !is_printing)
                .then(|| print_completion.lock().unwrap()(client, print_info));
        });
        MqttServer::new(mqtt.clone()).start_async(mqtt_listener)?;

        let http_listener = TcpListener::bind(addr(p_http)).context("Failed to bind HTTP")?;
        let http_port = http_listener.local_addr()?.port();
        let http = HttpServer::new(http_listener, &mqtt);
        http.start_async();

        let udp = UdpSocket::bind(addr(p_udp)).context("Failed to bind UDP")?;
        udp.set_read_timeout(Some(timeout))?;
        udp.set_broadcast(true)?;
        let _udp_port = udp.local_addr()?.port();

        info!("Binds: {{ UDP: {_udp_port}, MQTT: {mqtt_port}, HTTP: {http_port} }}");

        self.services = Some(Arc::new(Services {
            mqtt,
            http,
            udp,

            mqtt_port,
            http_port,
            udp_port: _udp_port,
        }));
        Ok(())
    }

    pub fn shutdown(&mut self) {
        if let Some(services) = self.services.take() {
            info!("Shutting down RemotePrintV1 services");
            services.http.shutdown();
            services.mqtt.shutdown();
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.services.is_some()
    }
}

impl Services {
    pub fn set_timeout(&self, timeout: Duration) -> Result<()> {
        self.udp.set_read_timeout(Some(timeout))?;
        Ok(())
    }

    fn connect_printer(
        &self,
        response: Response<FullStatusData>,
        address: SocketAddr,
    ) -> Result<()> {
        let machine_name = &response.data.attributes.machine_name;
        let mainboard_id = &response.data.attributes.mainboard_id;
        info!("Got status from `{machine_name}`",);

        if self.mqtt.clients.read().contains_key(mainboard_id) {
            warn!("Printer `{mainboard_id}` already connected.",);
            return Ok(());
        }

        // TODO ↓
        // if response.data.status.print_info.status == PrintInfoStatus::Complete {
        //     (print_completion.lock().sent).insert(response.data.attributes.mainboard_id.to_owned());
        // }

        self.mqtt.add_future_client(response);
        (self.udp)
            .send_to(format!("M66666 {}", self.mqtt_port).as_bytes(), address)
            .context("Failed to send mqtt connection command.")?;

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
        let response = serde_json::from_str::<Response<FullStatusData>>(&received)
            .context("Invalid response from printer.")?;

        self.connect_printer(response, address)?;
        Ok(())
    }

    pub fn scan(&self, broadcast: Ipv4Addr) -> Result<()> {
        info!("Scanning for printers on {broadcast}");
        (self.udp).send_to(b"M99999", SocketAddr::new(broadcast.into(), 3000))?;

        let mut buffer = [0; 1024];
        loop {
            let (len, addr) = match self.udp.recv_from(&mut buffer) {
                Ok(data) => data,
                Err(e) if matches!(e.kind(), ErrorKind::TimedOut | ErrorKind::WouldBlock) => break,
                Err(_) => continue,
            };

            let received = String::from_utf8_lossy(&buffer[..len]);
            let Ok(response) = serde_json::from_str::<Response<FullStatusData>>(&received) else {
                continue;
            };

            self.connect_printer(response, addr)?;
        }

        Ok(())
    }

    pub fn remove_printer(&self, mainboard: &str) -> Result<()> {
        self.mqtt.send_command(mainboard, DisconnectCommand)
    }

    pub fn upload(
        &self,
        mainboard: &str,
        data: Arc<Vec<u8>>,
        mut filename: String,
        format: RasterFormat,
    ) -> Result<()> {
        (!filename.is_empty()).then(|| filename.push('_'));
        filename.push_str(&random_string(8));
        filename.push('.');
        filename.push_str(format.extension());

        self.http.add_file(&filename, data.clone());

        let command = UploadFile::new(filename, self.http_port, &data);
        self.mqtt.send_command(mainboard, command)
    }

    pub fn print(&self, mainboard: &str, filename: &str) -> Result<()> {
        let command = StartPrinting {
            filename: filename.to_owned(),
            start_layer: 0,
        };
        self.mqtt.send_command(mainboard, command)
    }

    pub fn get_client(&self, mainboard: &str) -> MappedRwLockReadGuard<'_, MqttClient> {
        self.mqtt.get_client(mainboard)
    }

    pub fn clients(&self) -> RwLockReadGuard<'_, HashMap<String, MqttClient>> {
        self.mqtt.clients.read()
    }
}

impl Deref for RemotePrintV1 {
    type Target = Arc<Services>;

    fn deref(&self) -> &Self::Target {
        self.services.as_ref().unwrap()
    }
}
