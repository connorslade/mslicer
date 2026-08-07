use serde::{Deserialize, Serialize};

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
