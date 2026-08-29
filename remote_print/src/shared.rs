use std::net::{Ipv4Addr, SocketAddrV4};

use anyhow::Result;
use serde::{Deserialize, Deserializer, Serialize};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum PrintInfoStatus {
    None,
    InitialLower,
    Lowering,
    Exposure,
    Retracting,
    Pausing,
    Paused,
    Stopping,
    Stopped,
    Complete,
    CleckingFile,
    FinalRetract,
    Canceled, // maybe?
    Unknown(u8),
}

impl<'de> Deserialize<'de> for PrintInfoStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match u8::deserialize(deserializer)? {
            0 => Self::None,
            1 => Self::InitialLower,
            2 => Self::Lowering,
            3 => Self::Exposure,
            4 => Self::Retracting,
            5 => Self::Pausing,
            6 => Self::Paused,
            7 => Self::Stopping,
            8 | 14 => Self::Stopped, // 14 happens when the stop command is sent
            9 | 16 => Self::Complete,
            10 => Self::CleckingFile,
            12 => Self::FinalRetract,
            13 => Self::Canceled,
            other => Self::Unknown(other),
        })
    }
}

impl PrintInfoStatus {
    pub fn is_printing(&self) -> bool {
        !matches!(
            self,
            Self::None | Self::Complete | Self::Stopped | Self::Canceled
        )
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
