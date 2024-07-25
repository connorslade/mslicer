use std::{
    net::{IpAddr, SocketAddr, TcpListener, UdpSocket},
    str::FromStr,
    sync::Arc,
};

use anyhow::Result;
use common::misc::random_string;
use parking_lot::{Mutex, MutexGuard};
use tracing::info;

use remote_send::{
    commands::{DisconnectCommand, StartPrinting, UploadFile},
    http_server::HttpServer,
    mqtt::MqttServer,
    mqtt_server::Mqtt,
    status::FullStatusData,
    Response,
};

pub struct RemotePrint {
    services: Option<Services>,
    printers: Arc<Mutex<Vec<Printer>>>,
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

impl RemotePrint {
    pub fn uninitialized() -> Self {
        Self {
            services: None,
            printers: Arc::new(Mutex::new(Vec::new())),
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
        let _udp_port = udp.local_addr()?.port();

        info!("Binds: {{ UDP: {_udp_port}, MQTT: {mqtt_port}, HTTP: {http_port} }}");

        self.services = Some(Services {
            mqtt,
            http,
            udp,

            mqtt_port,
            http_port,
            _udp_port,
        });

        Ok(())
    }

    // todo: async
    pub fn add_printer(&mut self, address: &str) -> Result<()> {
        let services = self.services.as_ref().unwrap();

        let address = IpAddr::from_str(address)?;
        let address = SocketAddr::new(address, 3000);

        services.udp.send_to(b"M99999", address)?;

        let mut buffer = [0; 1024];
        let (len, _addr) = services.udp.recv_from(&mut buffer)?;

        let received = String::from_utf8_lossy(&buffer[..len]);
        let response = serde_json::from_str::<Response<FullStatusData>>(&received)?;
        info!(
            "Got status from `{}`",
            response.data.attributes.machine_name
        );

        self.printers.lock().push(Printer {
            mainboard_id: response.data.attributes.mainboard_id.clone(),
        });

        services.mqtt.add_future_client(response);

        services
            .udp
            .send_to(format!("M66666 {}", services.mqtt_port).as_bytes(), address)?;

        Ok(())
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
