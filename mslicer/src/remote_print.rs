use std::{
    io::ErrorKind,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, UdpSocket},
    str::FromStr,
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::{Context, Result};
use clone_macro::clone;
use common::misc::random_string;
use egui_modal::{Icon, Modal};
use parking_lot::{Mutex, MutexGuard};
use tracing::{info, warn};

use remote_send::{
    commands::{DisconnectCommand, StartPrinting, UploadFile},
    http_server::HttpServer,
    mqtt::MqttServer,
    mqtt_server::Mqtt,
    status::FullStatusData,
    Response,
};

use crate::app::{RemotePrintConnectStatus, UiState};

pub struct RemotePrint {
    services: Option<Arc<Services>>,
    printers: Arc<Mutex<Vec<Printer>>>,
    jobs: Vec<AsyncJob>,
}

struct Services {
    mqtt: Mqtt,
    http: HttpServer,
    udp: UdpSocket,

    mqtt_port: u16,
    http_port: u16,
    _udp_port: u16,
}

pub struct Printer {
    pub mainboard_id: String,
}

struct AsyncJob {
    handle: JoinHandle<Result<()>>,
    action: Box<dyn FnOnce(&mut UiState)>,
}

impl RemotePrint {
    pub fn uninitialized() -> Self {
        Self {
            services: None,
            printers: Arc::new(Mutex::new(Vec::new())),
            jobs: Vec::new(),
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.services.is_some()
    }

    pub fn printers(&self) -> MutexGuard<Vec<Printer>> {
        self.printers.lock()
    }

    pub fn mqtt(&self) -> &Mqtt {
        &self.services.as_ref().unwrap().mqtt
    }

    pub fn set_network_timeout(&self, timeout: Duration) {
        let services = self.services.as_ref().unwrap();
        services.udp.set_read_timeout(Some(timeout)).unwrap();
    }

    pub fn init(&mut self) -> Result<()> {
        info!("Starting remote print services");

        let mqtt_listener = TcpListener::bind("0.0.0.0:0")?;
        let mqtt_port = mqtt_listener.local_addr()?.port();
        let mqtt = Mqtt::new();
        MqttServer::new(mqtt.clone()).start_async(mqtt_listener)?;

        let http_listener = TcpListener::bind("0.0.0.0:0")?;
        let http_port = http_listener.local_addr()?.port();
        let http = HttpServer::new(http_listener);
        http.start_async();

        let udp = UdpSocket::bind("0.0.0.0:0")?;
        udp.set_broadcast(true)?;
        let _udp_port = udp.local_addr()?.port();

        info!("Binds: {{ UDP: {_udp_port}, MQTT: {mqtt_port}, HTTP: {http_port} }}");

        self.services = Some(Arc::new(Services {
            mqtt,
            http,
            udp,

            mqtt_port,
            http_port,
            _udp_port,
        }));

        Ok(())
    }

    pub fn tick(&mut self, modal: &mut Option<Modal>, ui_state: &mut UiState) {
        let mut dialog_builder = || modal.as_mut().unwrap().dialog();

        let mut i = 0;
        while i < self.jobs.len() {
            if self.jobs[i].is_finished() {
                let AsyncJob { handle, action } = self.jobs.remove(i);
                action(ui_state);
                if let Err(e) = handle.join().unwrap() {
                    let mut body = String::new();
                    for link in e.chain() {
                        body.push_str(&link.to_string());
                        body.push(' ');
                    }

                    dialog_builder()
                        .with_title("Remote Print Error")
                        .with_body(&body[..body.len() - 1])
                        .with_icon(Icon::Error)
                        .open();
                }
                continue;
            }

            i += 1;
        }
    }

    pub fn add_printer(&mut self, address: &str) -> Result<()> {
        let address = IpAddr::from_str(address)?;
        let address = SocketAddr::new(address, 3000);

        self.jobs.push(AsyncJob::new(
            thread::spawn(clone!(
                [{ self.printers } as printers, { self.services } as services],
                move || {
                    add_printer(services.unwrap(), printers, address)
                        .with_context(|| format!("Error adding printer at {}.", address.ip()))
                }
            )),
            |ui_state| {
                ui_state.remote_print_connecting = RemotePrintConnectStatus::None;
                ui_state.working_address.clear();
            },
        ));

        Ok(())
    }

    pub fn scan_for_printers(&mut self) {
        self.jobs.push(AsyncJob::new(
            thread::spawn(clone!(
                [{ self.printers } as printers, { self.services } as services],
                move || {
                    scan_for_printers(services.unwrap(), printers)
                        .context("Error scanning for printers.")
                }
            )),
            |ui_state| {
                ui_state.remote_print_connecting = RemotePrintConnectStatus::None;
            },
        ));
    }

    pub fn remove_printer(&mut self, index: usize) -> Result<()> {
        let services = self.services.as_ref().unwrap();
        let printer = self.printers.lock().remove(index);

        services
            .mqtt
            .send_command(&printer.mainboard_id, DisconnectCommand)?;

        Ok(())
    }

    pub fn upload(&self, mainboard_id: &str, data: Arc<Vec<u8>>) -> Result<()> {
        let services = self.services.as_ref().unwrap();

        let filename = format!("{}.goo", random_string(8));
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

impl AsyncJob {
    fn new(handle: JoinHandle<Result<()>>, action: impl FnOnce(&mut UiState) + 'static) -> Self {
        Self {
            handle,
            action: Box::new(action),
        }
    }

    fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }
}

fn add_printer(
    services: Arc<Services>,
    printers: Arc<Mutex<Vec<Printer>>>,
    address: SocketAddr,
) -> Result<()> {
    info!("Attempting to connect to printer at {}", address.ip());

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

fn scan_for_printers(services: Arc<Services>, printers: Arc<Mutex<Vec<Printer>>>) -> Result<()> {
    const BROADCAST: IpAddr = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 255));

    info!("Scanning for printers on {BROADCAST}");
    services
        .udp
        .send_to(b"M99999", SocketAddr::new(BROADCAST, 3000))?;

    let mut buffer = [0; 1024];
    loop {
        let (len, addr) = match services.udp.recv_from(&mut buffer) {
            Ok(data) => data,
            Err(e) if e.kind() == ErrorKind::TimedOut => break,
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

    printers.lock().push(Printer {
        mainboard_id: response.data.attributes.mainboard_id.clone(),
    });

    services.mqtt.add_future_client(response);

    services
        .udp
        .send_to(format!("M66666 {}", services.mqtt_port).as_bytes(), address)
        .context("Failed to send mqtt connection command.")?;

    Ok(())
}
