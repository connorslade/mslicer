use std::net::TcpStream;

use rand::{RngExt, rng};
use serde::Serialize;
use serde_json::json;
use tungstenite::WebSocket;

use crate::{
    shared::epoch,
    v3::status::{Command, CommandData},
};

pub fn send_command(socket: &mut WebSocket<TcpStream>, mainboard_id: &str, cmd: Cmd) {
    let message = serde_json::to_string(&Command {
        id: mainboard_id.to_owned(),
        data: CommandData {
            cmd: cmd.cmd(),
            data: cmd.data(),
            request_id: hex::encode(rng().random::<[u8; 8]>()),
            mainboard_id: mainboard_id.to_owned(),
            time_stamp: epoch(),
            from: 0,
        },
        topic: format!("sdcp/request/{mainboard_id}"),
    })
    .unwrap();
    socket
        .send(tungstenite::Message::Text(message.into()))
        .unwrap();
}

pub enum Cmd {
    RefreshStatus,
    RefreshAttributes,
    StartPrinting { filename: String, start_layer: u32 },
}

impl Cmd {
    pub fn cmd(&self) -> u8 {
        match self {
            Cmd::RefreshStatus => 0,
            Cmd::RefreshAttributes => 1,
            Cmd::StartPrinting { .. } => 128,
        }
    }

    pub fn data(&self) -> impl Serialize {
        match self {
            Cmd::RefreshStatus => json!({}),
            Cmd::RefreshAttributes => json!({}),
            Cmd::StartPrinting {
                filename,
                start_layer,
            } => json!({
                "Filename": filename,
                "StartLayer": start_layer
            }),
        }
    }
}
