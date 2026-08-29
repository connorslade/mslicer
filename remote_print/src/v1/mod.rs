use std::{
    collections::HashMap,
    net::{SocketAddr, TcpListener, UdpSocket},
    ops::Deref,
    sync::{Arc, atomic::Ordering},
};

use anyhow::{Context, Result};
use parking_lot::{Mutex, RwLockReadGuard};
use tracing::{info, warn};

use crate::{
    http::HttpServer,
    manager::Client,
    mqtt::MqttServer,
    shared::{Response, addr},
    v1::{
        commands::{DisconnectCommand, StartPrinting, StopPrinting, UploadFile},
        mqtt_server::{Mqtt, MqttClient},
        status::FullStatusData,
    },
};

pub mod commands;
pub mod mqtt_server;
pub mod status;

pub struct RemotePrintV1 {
    services: Option<Arc<Services>>,
}

pub struct Services {
    pub mqtt: Mqtt,
    pub http: HttpServer,

    pub mqtt_port: u16,
    pub http_port: u16,
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

    pub fn ports(&self) -> (u16, u16) {
        let services = &self.services();
        (services.mqtt_port, services.http_port)
    }

    pub fn init(
        &mut self,
        p_mqqt: u16,
        http: HttpServer,
        print_completion: impl FnMut(&Client) + Send + Sync + 'static,
    ) -> Result<()> {
        assert!(self.services.is_none());
        let print_completion = Mutex::new(print_completion);

        let mqtt_listener = TcpListener::bind(addr(p_mqqt)).context("Failed to bind MQTT")?;
        let mqtt_port = mqtt_listener.local_addr()?.port();
        let mqtt = Mqtt::new_callback(move |client| {
            let client_status = client.status.lock();
            let is_printing = client_status.print_info.status.is_printing();
            let was_printing = client.was_printing.swap(is_printing, Ordering::Relaxed);
            drop(client_status);

            let client = Client::from_v1(client);
            (was_printing && !is_printing).then(|| print_completion.lock()(&client));
        });
        MqttServer::new(mqtt.clone()).start_async(mqtt_listener)?;

        self.services = Some(Arc::new(Services {
            mqtt,
            http_port: http.port(),
            http,

            mqtt_port,
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
    pub(crate) fn connect_printer(
        &self,
        udp: &UdpSocket,
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

        self.mqtt.add_future_client(response);
        udp.send_to(format!("M66666 {}", self.mqtt_port).as_bytes(), address)
            .context("Failed to send mqtt connection command.")?;

        Ok(())
    }

    pub fn remove_printer(&self, mainboard: &str) -> Result<()> {
        self.mqtt.send_command(mainboard, DisconnectCommand)
    }

    pub fn upload(&self, mainboard: &str, data: Arc<Vec<u8>>, filename: String) -> Result<()> {
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

    pub fn stop_print(&self, mainboard: &str) -> Result<()> {
        self.mqtt.send_command(mainboard, StopPrinting)
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

impl Drop for RemotePrintV1 {
    fn drop(&mut self) {
        self.shutdown();
    }
}
