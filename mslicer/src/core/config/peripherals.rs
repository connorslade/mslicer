use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SpacenavConfig {
    pub gain: f32,
    pub rotation_gain: f32,
    pub position_gain: f32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RemotePrintConfig {
    pub init_at_startup: bool,
    pub status_proxy: bool,
    pub timeout: f32,

    pub broadcast_address: Ipv4Addr,
    pub mqtt_port: u16,
    pub http_port: u16,
    pub udp_port: u16,

    pub alert_completion: bool,
    pub webhook: Webhook,
}

#[derive(Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub enabled: bool,
    pub url: String,
    pub body: String,
    pub content_type: ContentType,
}

#[derive(Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    #[default]
    Text,
    Json,
}

impl ContentType {
    pub const ALL: &[Self] = &[Self::Text, Self::Json];

    pub fn name(&self) -> &str {
        match self {
            Self::Text => "Text",
            Self::Json => "JSON",
        }
    }

    pub fn header(&self) -> &str {
        match self {
            Self::Text => "text/plain; charset=utf-8",
            Self::Json => "application/json",
        }
    }
}

impl Default for SpacenavConfig {
    fn default() -> Self {
        Self {
            gain: 1.0,
            rotation_gain: 1.0,
            position_gain: 1.0,
        }
    }
}

impl Default for RemotePrintConfig {
    fn default() -> Self {
        Self {
            alert_completion: false,
            init_at_startup: false,
            status_proxy: false,
            timeout: 5.0,
            broadcast_address: Ipv4Addr::BROADCAST,
            mqtt_port: 0,
            http_port: 0,
            udp_port: 0,
            webhook: Webhook {
                enabled: false,
                url: String::new(),
                body: "Print %file% finished!".into(),
                content_type: ContentType::Text,
            },
        }
    }
}
