use std::{io::stdin, net::UdpSocket, thread};

use anyhow::Result;
use remote_send::{
    commands::StartPrinting, mqtt::MqttServer, mqtt_server::Mqtt, status::FullStatusData, Response,
};

const PRINTER_ADDRESS: &str = "192.168.1.233:3000";

fn main() -> Result<()> {
    let mqtt = Mqtt::new();
    let mqtt_inner = mqtt.clone();
    MqttServer::new(mqtt).start_async()?;

    let socket = UdpSocket::bind("0.0.0.0:3000")?;

    let msg = b"M99999";
    socket.send_to(msg, PRINTER_ADDRESS)?;

    let mut buffer = [0; 1024];
    let (len, _addr) = socket.recv_from(&mut buffer)?;

    let received = String::from_utf8_lossy(&buffer[..len]);
    let response = serde_json::from_str::<Response<FullStatusData>>(&received)?;
    println!(
        "Got status from `{}`",
        response.data.attributes.machine_name
    );
    let mainboard_id = response.data.attributes.mainboard_id.clone();
    mqtt_inner.add_future_client(response);

    let msg = b"M66666 1883";
    socket.send_to(msg, PRINTER_ADDRESS)?;

    let mut buf = String::new();
    stdin().read_line(&mut buf).unwrap();

    mqtt_inner
        .send_command(
            &mainboard_id,
            StartPrinting {
                filename: "out.goo".to_owned(),
                start_layer: 0,
            },
        )
        .unwrap();

    loop {
        thread::park()
    }
}
