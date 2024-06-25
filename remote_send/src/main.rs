use anyhow::Result;
use serde::Serialize;
use std::net::UdpSocket;

use remote_send::{
    mqtt::{
        packets::{
            connect::ConnectPacket,
            connect_ack::{ConnectAckFlags, ConnectAckPacket, ConnectReturnCode},
            publish::{PublishFlags, PublishPacket},
            publish_ack::PublishAckPacket,
            subscribe::SubscribePacket,
            subscribe_ack::{SubscribeAckPacket, SubscribeReturnCode},
        },
        HandlerCtx, MqttHandler, MqttServer,
    },
    status::{Attributes, FullStatusData, StatusData},
    Command, Response,
};

struct Mqtt {
    // todo: must support multiple clients
    status: Attributes,
    id: String,
}

impl MqttHandler for Mqtt {
    fn on_connect(
        &self,
        ctx: &HandlerCtx<Self>,
        _packet: ConnectPacket,
    ) -> Result<ConnectAckPacket> {
        println!("Client `{}` connected", ctx.client_id);

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

    fn on_publish_ack(&self, _ctx: &HandlerCtx<Self>, _packet: PublishAckPacket) -> Result<()> {
        Ok(())
    }

    fn on_disconnect(&self, _ctx: &HandlerCtx<Self>) -> Result<()> {
        Ok(())
    }
}

impl Mqtt {
    fn send_command<Data: Serialize>(
        &self,
        ctx: &HandlerCtx<Self>,
        cmd: u16,
        command: Data,
    ) -> Result<()> {
        let id = ctx.next_packet_id();

        let data = Command::new(cmd, command, self.id.to_owned());
        let data = serde_json::to_vec(&data).unwrap();

        ctx.server
            .send_packet(
                ctx.client_id,
                PublishPacket {
                    flags: PublishFlags::QOS1,
                    topic: format!("/sdcp/request/{}", self.status.mainboard_id),
                    packet_id: Some(id),
                    data,
                }
                .to_packet(),
            )
            .unwrap();

        Ok(())
    }
}

fn main() -> Result<()> {
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

    MqttServer::new(Mqtt {
        status: response.data.attributes,
        id: response.id,
    })
    .start_async()?;

    let msg = b"M66666 1883";
    socket.send_to(msg, "192.168.1.233:3000")?;

    Ok(())
}
