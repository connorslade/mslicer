use std::{
    io::ErrorKind,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, UdpSocket},
    sync::Arc,
    time::Duration,
};

use anyhow::{Context, Result};
use common::slice::format::RasterFormat;
use parking_lot::{Mutex, MutexGuard};
use tracing::{info, warn};

use remote_print::{
    Response,
    commands::{DisconnectCommand, StartPrinting, UploadFile},
    http_server::HttpServer,
    mqtt::MqttServer,
    mqtt_server::Mqtt,
    status::FullStatusData,
};

use crate::{
    app::App,
    app_ref_type,
    ui::popup::{Popup, PopupIcon},
    util::random_string,
};

pub struct RemotePrint {
    been_started: bool,
    pub services: Option<Arc<Services>>,
    pub printers: Arc<Mutex<Vec<Printer>>>,
}

app_ref_type!(RemotePrint, remote_print);

pub struct Services {
    mqtt: Mqtt,
    http: HttpServer,
    udp: UdpSocket,

    mqtt_port: u16,
    http_port: u16,
    udp_port: u16,
}

pub struct Printer {
    pub mainboard_id: String,
}

impl RemotePrint {
    pub fn uninitialized() -> Self {
        Self {
            been_started: false,
            services: None,
            printers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.services.is_some()
    }

    pub fn printers(&self) -> MutexGuard<'_, Vec<Printer>> {
        self.printers.lock()
    }

    pub fn ports(&self) -> Option<(u16, u16, u16)> {
        (self.services.as_ref()).map(|s| (s.mqtt_port, s.http_port, s.udp_port))
    }

    pub fn mqtt(&self) -> &Mqtt {
        &self.services.as_ref().unwrap().mqtt
    }

    pub fn http(&self) -> &HttpServer {
        &self.services.as_ref().unwrap().http
    }

    pub fn set_network_timeout(&self, timeout: Duration) {
        if let Some(services) = self.services.as_ref() {
            services.udp.set_read_timeout(Some(timeout)).unwrap();
        }
    }

    pub fn shutdown(&mut self) {
        self.been_started = false;
        if let Some(services) = self.services.take() {
            info!("Shutting down remote print services");
            services.http.shutdown();
            services.mqtt.shutdown();
        }
    }

    pub fn remove_printer(&mut self, index: usize) -> Result<()> {
        let services = self.services.as_ref().unwrap();
        let printer = self.printers.lock().remove(index);

        (services.mqtt).send_command(&printer.mainboard_id, DisconnectCommand)?;
        Ok(())
    }

    pub fn upload(
        &self,
        mainboard_id: &str,
        data: Arc<Vec<u8>>,
        mut filename: String,
        format: RasterFormat,
    ) -> Result<()> {
        let services = self.services.as_ref().unwrap();

        if !filename.is_empty() {
            filename.push('_');
        }
        filename.push_str(&random_string(8));
        filename.push('.');
        filename.push_str(format.extension());

        services.http.add_file(&filename, data.clone());

        services
            .mqtt
            .send_command(
                mainboard_id,
                UploadFile::new(filename, services.http_port, &data),
            )
            .unwrap();

        Ok(())
    }

    pub fn print(&self, mainboard_id: &str, filename: &str) -> Result<()> {
        let services = self.services.as_ref().unwrap();

        services
            .mqtt
            .send_command(
                mainboard_id,
                StartPrinting {
                    filename: filename.to_owned(),
                    start_layer: 0,
                },
            )
            .unwrap();

        Ok(())
    }
}

impl<'a> RemotePrintRef<'a> {
    pub fn init(&mut self) {
        if let Err(e) = self._init() {
            self.app.popup.open(Popup::simple(
                "Failed to Start Remote Print",
                PopupIcon::Error,
                e.to_string(),
            ));
        }
    }

    fn _init(&mut self) -> Result<()> {
        info!("Starting remote print services");
        let addr = |port| SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);
        let config = &self.app.config.remote_print;

        let mqtt_listener =
            TcpListener::bind(addr(config.mqtt_port)).context("Failed to bind MQTT")?;
        let mqtt_port = mqtt_listener.local_addr()?.port();
        let mqtt = Mqtt::new();
        MqttServer::new(mqtt.clone()).start_async(mqtt_listener)?;

        let http_listener =
            TcpListener::bind(addr(config.http_port)).context("Failed to bind HTTP")?;
        let http_port = http_listener.local_addr()?.port();
        let http = HttpServer::new(http_listener, &mqtt);
        http.start_async();

        let udp = UdpSocket::bind(addr(config.udp_port)).context("Failed to bind UDP")?;
        udp.set_broadcast(true)?;
        let _udp_port = udp.local_addr()?.port();

        info!("Binds: {{ UDP: {_udp_port}, MQTT: {mqtt_port}, HTTP: {http_port} }}");

        self.been_started = true;
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

    pub fn tick(&mut self) {
        if !self.is_initialized() && self.app.config.remote_print.init_at_startup {
            self.init();
        }

        if !self.been_started && self.is_initialized() {
            self.been_started = true;

            let config = &self.app.config.remote_print;
            let services = self.services.as_ref().unwrap();
            self.set_network_timeout(Duration::from_secs_f32(config.timeout));
            services.http.set_proxy_enabled(config.status_proxy);
        }
    }
}

pub fn add_printer(
    services: Arc<Services>,
    printers: Arc<Mutex<Vec<Printer>>>,
    address: Ipv4Addr,
) -> Result<()> {
    info!("Attempting to connect to printer at {address}");
    let address = SocketAddr::new(address.into(), 3000);

    services.udp.send_to(b"M99999", address)?;

    let mut buffer = [0; 1024];
    let (len, _addr) = services
        .udp
        .recv_from(&mut buffer)
        .context("No response from printer.")?;

    let received = String::from_utf8_lossy(&buffer[..len]);
    let response = serde_json::from_str::<Response<FullStatusData>>(&received)
        .context("Invalid response from printer.")?;

    connect_printer(services, printers, response, address)?;
    Ok(())
}

pub fn scan_for_printers(
    services: Arc<Services>,
    printers: Arc<Mutex<Vec<Printer>>>,
    broadcast: Ipv4Addr,
) -> Result<()> {
    info!("Scanning for printers on {broadcast}");
    (services.udp).send_to(b"M99999", SocketAddr::new(broadcast.into(), 3000))?;

    let mut buffer = [0; 1024];
    loop {
        let (len, addr) = match services.udp.recv_from(&mut buffer) {
            Ok(data) => data,
            Err(e) if matches!(e.kind(), ErrorKind::TimedOut | ErrorKind::WouldBlock) => break,
            Err(_) => continue,
        };

        let received = String::from_utf8_lossy(&buffer[..len]);
        let Ok(response) = serde_json::from_str::<Response<FullStatusData>>(&received) else {
            continue;
        };

        if let Err(err) = connect_printer(services.clone(), printers.clone(), response, addr) {
            warn!("Failed to connect to printer while scanning: {}", err);
        };
    }

    Ok(())
}

fn connect_printer(
    services: Arc<Services>,
    printers: Arc<Mutex<Vec<Printer>>>,
    response: Response<FullStatusData>,
    address: SocketAddr,
) -> Result<()> {
    info!(
        "Got status from `{}`",
        response.data.attributes.machine_name
    );

    let mut printers = printers.lock();
    if printers
        .iter()
        .any(|p| p.mainboard_id == response.data.attributes.mainboard_id)
    {
        warn!(
            "Printer `{}` already connected.",
            response.data.attributes.mainboard_id
        );
        return Ok(());
    }

    printers.push(Printer {
        mainboard_id: response.data.attributes.mainboard_id.clone(),
    });

    services.mqtt.add_future_client(response);

    services
        .udp
        .send_to(format!("M66666 {}", services.mqtt_port).as_bytes(), address)
        .context("Failed to send mqtt connection command.")?;

    Ok(())
}
