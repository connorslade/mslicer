use std::{
    fs,
    net::{IpAddr, Ipv4Addr},
    path::{Path, PathBuf},
};

use anyhow::Result;
use egui::Theme;
use egui_dock::Tree;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::{render::pipelines::model::RenderStyle, windows::Tab};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub render_style: RenderStyle,
    pub grid_size: f32,
    pub theme: Theme,
    pub recent_projects: Vec<PathBuf>,
    pub panels: Option<Tree<Tab>>,

    // Remote print settings
    pub alert_print_completion: bool,
    pub init_remote_print_at_startup: bool,
    pub http_status_proxy: bool,
    pub network_timeout: f32,
    pub network_broadcast_address: IpAddr,

    // Advanced Settings
    pub show_normals: bool,
    pub max_buffer_size: u64,
}

impl Config {
    pub fn load_or_default(config_dir: &Path) -> Self {
        match Self::load(config_dir) {
            Ok(config) => config,
            Err(err) => {
                warn!("Failed to load config, using defaults: {}", err);
                Config::default()
            }
        }
    }

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
            render_style: RenderStyle::Rendered,
            grid_size: 12.16,
            theme: Theme::Dark,

            recent_projects: Vec::new(),
            panels: None,

            alert_print_completion: false,
            init_remote_print_at_startup: false,
            http_status_proxy: false,
            network_timeout: 5.0,
            network_broadcast_address: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 255)),

            max_buffer_size: 512 << 20,
            show_normals: false,
        }
    }
}
