use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

use crate::shared::{FileTransferInfo, PrintInfo, Resolution, parse_resolution};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct FullStatusData {
    pub attributes: Attributes,
    pub status: Status,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct StatusData {
    pub status: Status,
    #[serde(rename = "MainboardID")]
    pub mainboard_id: String,
    pub time_stamp: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Attributes {
    pub name: String,
    pub machine_name: String,
    pub protocol_version: String,
    pub firmware_version: String,
    #[serde(deserialize_with = "parse_resolution")]
    pub resolution: Resolution,
    #[serde(rename = "MainboardIP")]
    pub mainboard_ip: String,
    #[serde(rename = "MainboardID")]
    pub mainboard_id: String,
    #[serde(rename = "SDCPStatus")]
    pub sdcp_status: u8,
    #[serde(rename = "LocalSDCPAddress")]
    pub local_sdcp_address: String,
    #[serde(rename = "SDCPAddress")]
    pub sdcp_address: String,
    pub capabilities: Vec<Capability>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Capability {
    FileTransfer,
    PrintControl,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Status {
    pub current_status: CurrentStatus,
    pub previous_status: u8,
    pub print_info: PrintInfo,
    pub file_transfer_info: FileTransferInfo,
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize_repr, Serialize)]
pub enum CurrentStatus {
    Ready = 0,
    Busy = 1,
    TransferringFile = 2,
}
