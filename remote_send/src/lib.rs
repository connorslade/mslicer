use anyhow::Result;
use serde::{Deserialize, Deserializer, Serialize};

pub mod commands;
pub mod http_server;
pub mod mqtt;
pub mod mqtt_server;
pub mod status;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Response<Data> {
    pub id: String,
    pub data: Data,
}

#[derive(Debug, Clone, Serialize)]
pub struct Resolution {
    pub x: u16,
    pub y: u16,
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
