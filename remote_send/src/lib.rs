use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Deserializer, Serialize};

pub mod mqtt;
pub mod status;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Response<Data> {
    pub id: String,
    pub data: Data,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Command<Data> {
    pub cmd: u16,
    pub data: Data,
    pub from: u8,
    #[serde(rename = "MainboardID")]
    pub mainboard_id: String,
    #[serde(rename = "RequestID")]
    pub request_id: String,
    #[serde(rename = "TimeStamp")]
    pub time_stamp: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct StartPrinting {
    pub filename: String,
    pub start_layer: u32,
}

impl<Data> Command<Data> {
    pub fn new(cmd: u16, data: Data, mainboard_id: String) -> Self {
        let request_id = format!("{:x}", rand::random::<u128>());
        let time_stamp = Utc::now().timestamp_millis() as u64;

        Self {
            cmd,
            data,
            from: 0,
            mainboard_id,
            request_id,
            time_stamp,
        }
    }
}

#[derive(Debug)]
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
