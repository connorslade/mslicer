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

// // https://github.com/danielcherubini/elegoo-homeassistant/blob/83db65f58d8d8b6d4575c5d9c5d7d48ddd7fe37f/custom_components/elegoo_printer/websocket/server/discovery.py#L100-L101
// #[derive(Debug, Clone, Deserialize, Serialize)]
// #[serde(rename_all = "PascalCase")]
// pub struct LegacyDiscoveryResponse {
//     pub name: String,
//     pub machine_name: String,
//     pub brand_name: String,
//     #[serde(rename = "MainboardIP")]
//     pub mainboard_ip: String,
//     #[serde(rename = "MainboardID")]
//     pub mainboard_id: String,
//     pub protocol_version: String,
//     pub firmware_version: String,
// }

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

/*
* ts = int(time.time())
        data = data or {}
        request_id = secrets.token_hex(8)
        payload = {
            "Id": self.printer.connection,
            "Data": {
                "Cmd": cmd,
                "Data": data,
                "RequestID": request_id,
                "MainboardID": self.printer.id,
                "TimeStamp": ts,
                "From": 0,
            },
            "Topic": f"sdcp/request/{self.printer.id}",
        }
*/

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Command<T> {
    pub id: String,
    pub data: CommandData<T>,
    pub topic: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CommandData<T> {
    pub cmd: u8,
    pub data: T,
    #[serde(rename = "RequestID")]
    pub request_id: String,
    #[serde(rename = "MainboardID")]
    pub mainboard_id: String,
    pub time_stamp: i64,
    pub from: u8,
}
