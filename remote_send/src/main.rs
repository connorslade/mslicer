use anyhow::Result;
use std::{net::UdpSocket, thread};

use remote_send::{
    mqtt::{
        packets::{
            connect::ConnectPacket,
            connect_ack::{ConnectAckFlags, ConnectAckPacket, ConnectReturnCode},
            publish::{PublishFlags, PublishPacket},
            subscribe::SubscribePacket,
            subscribe_ack::{SubscribeAckPacket, SubscribeReturnCode},
        },
        HandlerCtx, MqttHandler, MqttServer,
    },
    status::{FullStatusData, StatusData},
    Response,
};

struct Mqtt;

impl MqttHandler for Mqtt {
    fn on_connect(
        &self,
        ctx: &HandlerCtx<Self>,
        _packet: ConnectPacket,
    ) -> Result<ConnectAckPacket> {
        println!("Client `{}` connected", ctx.client_id);

        let id = ctx.next_packet_id();
        let (client_id, server) = (ctx.client_id, ctx.server.clone());
        thread::spawn(move || {
            thread::sleep(std::time::Duration::from_secs(5));
            println!("Sending command to client `{}` (id: {id})", client_id);

            server
                .send_packet(
                    client_id,
                    PublishPacket {
                        flags: PublishFlags::QOS1,
                        topic: "/sdcp/request/0001D2635E0347DA".to_owned(),
                        packet_id: Some(id),
                        data: br#"{
    "Data": {
        "Cmd": 128,
        "Data": {
            "Filename": "out.goo",
            "StartLayer": 0
        },
        "From": 0,
        "MainboardID": "0001D2635E0347DA",
        "RequestID": "b353f511680d40278c48602821a9e6ec",
        "TimeStamp": 1719258574000
    },
    "Id": "0a69ee780fbd40d7bfb95b312250bf46"
}"#
                        .to_vec(),
                    }
                    .to_packet(),
                )
                .unwrap();
        });

        Ok(ConnectAckPacket {
            flags: ConnectAckFlags::empty(),
            return_code: ConnectReturnCode::Accepted,
        })
    }

    fn on_subscribe(
        &self,
        ctx: &HandlerCtx<Self>,
        packet: SubscribePacket,
    ) -> Result<SubscribeAckPacket> {
        println!(
            "Client `{}` subscribed to topics: {:?}",
            ctx.client_id, packet.filters
        );

        Ok(SubscribeAckPacket {
            packet_id: packet.packet_id,
            return_codes: packet
                .filters
                .iter()
                .map(|(_, qos)| SubscribeReturnCode::Success(*qos))
                .collect::<Vec<_>>(),
        })
    }

    fn on_publish(&self, ctx: &HandlerCtx<Self>, packet: PublishPacket) -> Result<()> {
        println!(
            "Client `{}` published to topic `{}`",
            ctx.client_id, packet.topic
        );

        if let Some(board_id) = packet.topic.strip_prefix("/sdcp/status/") {
            let status = serde_json::from_slice::<Response<StatusData>>(&packet.data)?;
            println!("Got status from `{}`", board_id);
            println!("{:?}", status);
        } else if let Some(board_id) = packet.topic.strip_prefix("/sdcp/response/") {
            println!("Got command response from `{}`", board_id);
            println!("{:?}", String::from_utf8_lossy(&packet.data));
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    MqttServer::new(Mqtt).start_async()?;

    let socket = UdpSocket::bind("0.0.0.0:3000")?;

    // let msg = b"M99999";
    let msg = b"M66666 1883";
    socket.send_to(msg, "192.168.1.233:3000")?;

    let mut buffer = [0; 1024];
    let (len, _addr) = socket.recv_from(&mut buffer)?;

    let received = String::from_utf8_lossy(&buffer[..len]);
    let response = serde_json::from_str::<Response<FullStatusData>>(&received)?;
    println!(
        "Got status from `{}`",
        response.data.attributes.machine_name
    );

    Ok(())
}
