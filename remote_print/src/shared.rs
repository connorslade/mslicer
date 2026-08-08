use std::net::{Ipv4Addr, SocketAddrV4};

use anyhow::Result;
use serde::{Deserialize, Deserializer, Serialize};
use serde_repr::Deserialize_repr;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Response<Data> {
    pub id: String,
    pub data: Data,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PrintInfo {
    pub status: PrintInfoStatus,
    pub current_layer: u32,
    pub total_layer: u32,
    pub current_ticks: u32,
    pub total_ticks: u32,
    pub error_number: u8,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Resolution {
    pub x: u16,
    pub y: u16,
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize_repr, Serialize)]
pub enum PrintInfoStatus {
    None = 0,
    InitialLower = 1,
    Lowering = 2,
    Exposure = 3,
    Retracting = 4,
    Pausing = 5,
    Paused = 6,
    Stopping = 7,
    Stopped = 8,
    Complete2 = 9, // todo fix
    CleckingFile = 10,
    FinalRetract = 12,
    Canceled = 13, // maybe?
    Complete = 16,
}

impl PrintInfoStatus {
    pub fn is_printing(&self) -> bool {
        !matches!(self, Self::None | Self::Complete | Self::Complete2)
    }
}

pub fn parse_resolution<'de, D>(from: D) -> Result<Resolution, D::Error>
where
    D: Deserializer<'de>,
{
    let str = String::deserialize(from)?;
    let (x, y) = str
        .split_once('x')
        .ok_or_else(|| serde::de::Error::custom("Invalid resolution"))?;
    Ok(Resolution {
        x: x.parse().map_err(serde::de::Error::custom)?,
        y: y.parse().map_err(serde::de::Error::custom)?,
    })
}

pub fn addr(port: u16) -> SocketAddrV4 {
    SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port)
}

pub fn epoch() -> i64 {
    chrono::Utc::now().timestamp()
}
