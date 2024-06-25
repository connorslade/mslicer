use chrono::Utc;
use serde::Serialize;

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

pub trait CommandTrait: Serialize {
    const CMD: u16;
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct StartPrinting {
    pub filename: String,
    pub start_layer: u32,
}

impl CommandTrait for StartPrinting {
    const CMD: u16 = 128;
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UploadFile {
    pub check: u8,
    pub clean_cache: u8,
    pub compress: u8,
    pub file_size: u32,
    pub filename: String,
    pub md5: String,
    pub url: String,
}

impl CommandTrait for UploadFile {
    const CMD: u16 = 256;
}

#[derive(Serialize)]
pub struct DisconnectCommand;

impl CommandTrait for DisconnectCommand {
    const CMD: u16 = 64;
}
