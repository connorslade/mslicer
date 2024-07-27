use std::{
    fs,
    net::{IpAddr, Ipv4Addr},
    path::Path,
};

use anyhow::Result;
use eframe::Theme;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::render::pipelines::model::RenderStyle;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub render_style: RenderStyle,
    pub grid_size: f32,
    pub theme: Theme,
    pub alert_print_completion: bool,
    pub init_remote_print_at_startup: bool,
    pub network_timeout: f32,
    pub network_broadcast_address: IpAddr,
}

impl Config {
    pub fn load(config_dir: &Path) -> Result<Self> {
        let config_file = config_dir.join("config.toml");
        Ok(if config_file.exists() {
            let file = fs::read(&config_file)?;
            let string = String::from_utf8_lossy(&file);
            let config = toml::from_str(&string)?;
            info!("Successfully loaded config file");
            config
        } else {
            info!("No config file found, using defaults");
            Self::default()
        })
    }

    pub fn save(&self, config_dir: &Path) -> Result<()> {
        fs::create_dir_all(config_dir)?;

        let config_file = config_dir.join("config.toml");
        let string = toml::to_string(self)?;
        fs::write(config_file, string)?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            render_style: RenderStyle::Rended,
            theme: Theme::Dark,
            grid_size: 12.16,
            alert_print_completion: false,
            init_remote_print_at_startup: false,
            network_timeout: 5.0,
            network_broadcast_address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 255)),
        }
    }
}
