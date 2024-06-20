use serde::Deserialize;

use crate::{parse_resolution, Resolution};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StatusData {
    pub attributes: Attributes,
    pub status: Status,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Attributes {
    name: String,
    machine_name: String,
    protocol_version: String,
    firmware_version: String,
    #[serde(deserialize_with = "parse_resolution")]
    resolution: Resolution,
    #[serde(rename = "MainboardIP")]
    mainboard_ip: String,
    #[serde(rename = "MainboardID")]
    mainboard_id: String,
    #[serde(rename = "SDCPStatus")]
    sdcp_status: u8,
    #[serde(rename = "LocalSDCPAddress")]
    local_sdcp_address: String,
    #[serde(rename = "SDCPAddress")]
    sdcp_address: String,
    capabilities: Vec<Capability>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Capability {
    FileTransfer,
    PrintControl,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Status {
    current_status: u8,
    previous_status: u8,
    print_info: PrintInfo,
    file_transfer_info: FileTransferInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PrintInfo {
    status: u8,
    current_layer: u32,
    total_layer: u32,
    current_ticks: u32,
    total_ticks: u32,
    error_number: u8,
    filename: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FileTransferInfo {
    status: u8,
    download_offset: u32,
    check_offset: u32,
    file_total_size: u32,
    filename: String,
}
