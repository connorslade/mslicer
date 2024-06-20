use std::net::UdpSocket;

use anyhow::Result;

use remote_send::{status::StatusData, Response};

fn main() -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:3000")?;

    let msg = b"M99999";
    socket.send_to(msg, "192.168.1.233:3000")?;

    let mut buffer = [0; 1024];
    let (len, _addr) = socket.recv_from(&mut buffer)?;

    let received = String::from_utf8_lossy(&buffer[..len]);
    let response = serde_json::from_str::<Response<StatusData>>(&received)?;
    println!("Got status from `{}`", response.data.attributes.machine_name);

    Ok(())
}
