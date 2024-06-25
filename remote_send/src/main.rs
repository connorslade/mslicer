use std::{net::UdpSocket, thread};

use anyhow::Result;
use remote_send::{mqtt::MqttServer, mqtt_server::Mqtt, status::FullStatusData, Response};

fn main() -> Result<()> {
    let mqtt = Mqtt::new();
    let mqtt_inner = mqtt.clone();
    MqttServer::new(mqtt).start_async()?;

    let socket = UdpSocket::bind("0.0.0.0:3000")?;

    let msg = b"M99999";
    socket.send_to(msg, "192.168.1.233:3000")?;

    let mut buffer = [0; 1024];
    let (len, _addr) = socket.recv_from(&mut buffer)?;

    let received = String::from_utf8_lossy(&buffer[..len]);
    let response = serde_json::from_str::<Response<FullStatusData>>(&received)?;
    println!(
        "Got status from `{}`",
        response.data.attributes.machine_name
    );
    mqtt_inner.add_future_client(response.data.attributes, response.id);

    let msg = b"M66666 1883";
    socket.send_to(msg, "192.168.1.233:3000")?;

    loop {
        thread::park()
    }
}
