use std::{
    fs,
    io::stdin,
    net::{TcpListener, UdpSocket},
    sync::Arc,
    thread,
};

use anyhow::Result;
use remote_send::{
    Response,
    commands::{StartPrinting, UploadFile},
    http_server::HttpServer,
    mqtt::MqttServer,
    mqtt_server::Mqtt,
    status::FullStatusData,
};

const PRINTER_ADDRESS: &str = "192.168.1.233:3000";

fn main() -> Result<()> {
    let mqtt_listener = TcpListener::bind("0.0.0.0:0")?;
    let mqtt_port = mqtt_listener.local_addr()?.port();
    let mqtt = Mqtt::new();
    MqttServer::new(mqtt.clone()).start_async(mqtt_listener)?;

    let http_listener = TcpListener::bind("0.0.0.0:0")?;
    let http_port = http_listener.local_addr()?.port();
    let http = HttpServer::new(http_listener, &mqtt);
    http.start_async();

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    let socket_port = socket.local_addr()?.port();

    println!("Binds: {{ UDP: {socket_port}, MQTT: {mqtt_port}, HTTP: {http_port} }}");

    socket.send_to(b"M99999", PRINTER_ADDRESS)?;

    let mut buffer = [0; 1024];
    let (len, _addr) = socket.recv_from(&mut buffer)?;

    let received = String::from_utf8_lossy(&buffer[..len]);
    let response = serde_json::from_str::<Response<FullStatusData>>(&received)?;
    println!(
        "Got status from `{}`",
        response.data.attributes.machine_name
    );
    let mainboard_id = response.data.attributes.mainboard_id.clone();
    mqtt.add_future_client(response);

    socket.send_to(format!("M66666 {mqtt_port}").as_bytes(), PRINTER_ADDRESS)?;

    // wait for user to press enter
    let mut buf = String::new();
    stdin().read_line(&mut buf).unwrap();

    let teapot = Arc::new(fs::read("fox.goo")?);
    http.add_file("fox.goo", teapot.clone());

    mqtt.send_command(
        &mainboard_id,
        UploadFile::new("fox.goo".to_owned(), http_port, &teapot),
    )
    .unwrap();

    let mut buf = String::new();
    stdin().read_line(&mut buf).unwrap();

    mqtt.send_command(
        &mainboard_id,
        StartPrinting {
            filename: "fox.goo".to_owned(),
            start_layer: 0,
        },
    )
    .unwrap();

    loop {
        thread::park()
    }
}
