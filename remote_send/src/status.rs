use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

use crate::{parse_resolution, Resolution};

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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct FileTransferInfo {
    pub status: FileTransferStatus,
    pub download_offset: u32,
    pub check_offset: u32,
    pub file_total_size: u32,
    pub filename: String,
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize_repr, Serialize)]
pub enum CurrentStatus {
    Ready = 0,
    Busy = 1,
    TransferringFile = 2,
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize_repr, Serialize)]
pub enum PrintInfoStatus {
    None = 0,
    InitialLower = 1,
    Lowering = 2,
    Exposure = 3,
    Retracting = 4,
    FinalRetract = 12,
    Complete = 16,
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize_repr, Serialize)]
pub enum FileTransferStatus {
    None = 0,
    Done = 2,
    Error = 3,
}
