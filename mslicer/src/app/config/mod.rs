use std::{
    fs, iter,
    path::{Path, PathBuf},
};

use anyhow::Result;
use common::slice::SliceConfig;
use egui::Theme;
use egui_dock::Tree;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::{
    app::config::{
        peripherals::{RemotePrintConfig, SpacenavConfig},
        printers::PrinterProperties,
        render::RenderConfig,
        sliced::SlicedConfig,
    },
    windows::Tab,
};

pub mod peripherals;
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
            tasks: true,
        }
    }
}
