use std::{
    borrow::Cow,
    fs, iter,
    net::Ipv4Addr,
    path::{Path, PathBuf},
};

use anyhow::Result;
use common::{slice::SliceConfig, units::Milimeters};
use egui::Theme;
use egui_dock::Tree;
use itertools::Itertools;
use nalgebra::{Vector2, Vector3};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::{
    app::config::{printers::PrinterProperties, render::RenderConfig, sliced::SlicedConfig},
    windows::Tab,
};

pub mod printers;
pub mod render;
pub mod sliced;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub ui: UiConfig,
    pub render: RenderConfig,
    pub sliced: SlicedConfig,
    pub spacenav: SpacenavConfig,
    pub remote_print: RemotePrintConfig,
    pub default_slice_config: SliceConfig,

    pub recent_projects: Vec<PathBuf>,
    pub printers: Vec<PrinterProperties>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub theme: Theme,
    pub panels: Option<Tree<Tab>>,
    pub about: bool,
    pub tasks: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SpacenavConfig {
    pub gain: f32,
    pub rotation_gain: f32,
    pub position_gain: f32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RemotePrintConfig {
    pub init_at_startup: bool,
    pub status_proxy: bool,
    pub timeout: f32,

    pub broadcast_address: Ipv4Addr,
    pub mqtt_port: u16,
    pub http_port: u16,
    pub udp_port: u16,

    pub alert_completion: bool,
    pub webhook: Webhook,
}

#[derive(Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub enabled: bool,
    pub url: String,
    pub body: String,
    pub content_type: ContentType,
}

#[derive(Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    #[default]
    Text,
    Json,
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

impl Config {
    pub fn add_recent_project(&mut self, path: PathBuf) {
        self.recent_projects = iter::once(path)
            .chain(self.recent_projects.iter().cloned())
            .unique()
            .take(5)
            .collect()
    }
}

impl ContentType {
    pub const ALL: &[Self] = &[Self::Text, Self::Json];

    pub fn name(&self) -> &str {
        match self {
            Self::Text => "Text",
            Self::Json => "JSON",
        }
    }

    pub fn header(&self) -> &str {
        match self {
            Self::Text => "text/plain; charset=utf-8",
            Self::Json => "application/json",
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ui: Default::default(),
            render: Default::default(),
            sliced: Default::default(),
            spacenav: Default::default(),
            remote_print: Default::default(),
            default_slice_config: Default::default(),

            recent_projects: Vec::new(),
            printers: vec![PrinterProperties::new(
                "Custom Printer",
                [11_520, 5_120],
                [218.88, 122.904, 260.0],
            )],
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            panels: None,
            about: true,
            tasks: false,
        }
    }
}

impl Default for SpacenavConfig {
    fn default() -> Self {
        Self {
            gain: 1.0,
            rotation_gain: 1.0,
            position_gain: 1.0,
        }
    }
}

impl Default for RemotePrintConfig {
    fn default() -> Self {
        Self {
            alert_completion: false,
            init_at_startup: false,
            status_proxy: false,
            timeout: 5.0,
            broadcast_address: Ipv4Addr::BROADCAST,
            mqtt_port: 0,
            http_port: 0,
            udp_port: 0,
            webhook: Webhook {
                enabled: false,
                url: String::new(),
                body: "Print %file% finished!".into(),
                content_type: ContentType::Text,
            },
        }
    }
}

impl Default for PrinterProperties {
    fn default() -> Self {
        Self {
            name: Cow::Owned("New Printer".into()),
            resolution: Vector2::new(10_000, 5_000),
            size: Vector3::repeat(100.0).map(Milimeters::new),
        }
    }
}
