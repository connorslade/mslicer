use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

use crate::shared::PrintInfo;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Message {
    #[serde(rename = "MainboardID")]
    pub mainboard_id: String,
    pub time_stamp: u64,
    pub topic: String,

    pub attributes: Option<Attributes>,
    pub status: Option<Status>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DiscoveryResponse {
    pub name: String,
    pub machine_name: String,
    pub brand_name: String,
    #[serde(rename = "MainboardIP")]
    pub mainboard_ip: String,
    #[serde(rename = "MainboardID")]
    pub mainboard_id: String,
    pub protocol_version: String,
    pub firmware_version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Attributes {
    pub name: String,
    pub machine_name: String,
    pub brand_name: String,
    pub protocol_version: String,
    pub firmware_version: String,
    pub capabilities: Vec<String>,
    pub support_file_type: Vec<String>,
    pub resolution: String,
    #[serde(rename = "XYZsize")]
    pub xyz_size: String,
    #[serde(rename = "MainboardIP")]
    pub mainboard_ip: String,
    #[serde(rename = "MainboardID")]
    pub mainboard_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Status {
    pub current_status: Vec<CurrentStatus>,
    pub print_info: PrintInfo,
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize_repr, Serialize)]
pub enum CurrentStatus {
    Idle = 0,
    Printing = 1,
    FileTransferring = 2,
    ExposureTesting = 3,
    DevicesTesting = 4,
}
