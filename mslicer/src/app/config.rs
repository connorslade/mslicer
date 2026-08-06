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
    render::{camera::Projection, workspace::model::RenderStyle},
    windows::Tab,
};

#[rustfmt::skip]
pub const DEFAULT_PRINTERS: &[(&str, &[PrinterProperties])] = &[
    ("Elegoo", &[
        PrinterProperties::new("Saturn 3",              [11_520, 5_120], [218.88,  122.88,  250.0]),
        PrinterProperties::new("Saturn 3 Ultra",        [11_520, 5_120], [218.88,  122.904, 260.0]),
        PrinterProperties::new("Saturn 4",              [11_520, 5_120], [218.88,  122.88,  220.0]),
        PrinterProperties::new("Saturn 4 Ultra",        [11_520, 5_120], [218.88,  122.88,  220.0]),
        PrinterProperties::new("Saturn 4 Ultra 16K",    [15_120, 6_230], [211.68,  118.37,  220.0]),
        PrinterProperties::new("Jupiter SE",            [5_448,  3_064], [277.848, 156.264, 300.0]),
        PrinterProperties::new("Jupiter 2",             [15_120, 6_230], [302.0,   162.0,   300.0]),
        PrinterProperties::new("Mars 5",                [4_098,  2_560], [143.43,  89.6,    150.0]),
        PrinterProperties::new("Mars 5 Ultra",          [8_520,  4_320], [153.36,  77.76,   165.0]),
        PrinterProperties::new("Mars 4",                [8_520,  4_320], [153.36,  77.76,   175.0]),
        PrinterProperties::new("Mars 4 Ultra",          [8_520,  4_320], [153.36,  77.76,   165.0]),
    ]),
    ("Phrozen", &[
        PrinterProperties::new("Sonic Mini 4K",         [3_840,  2_160], [134.40,  75.600,  130.0]),
        PrinterProperties::new("Sonic Mini 8K",         [7_500,  3_240], [165.00,  71.280,  180.0]),
        PrinterProperties::new("Sonic Mega 8K",         [7_680,  4_320], [330.24,  185.76,  400.0]),
        PrinterProperties::new("Sonic Mega 8K V2",      [7_680,  4_320], [330.24,  185.76,  400.0]),
        PrinterProperties::new("Sonic Mighty 8K",       [7_680,  4_320], [218.88,  123.12,  235.0]),
        PrinterProperties::new("Sonic Mighty 12K",      [11_520, 5_120], [218.88,  123.12,  235.0]),
        PrinterProperties::new("Sonic Mighty Revo",     [13_320, 5_120], [223.78,  126.98,  235.0]),
        PrinterProperties::new("Sonic Mighty Revo 16K", [15_120, 6_230], [211.68,  118.37,  235.0]), // verify!
    ])
];

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub grid_size: f32,
    pub theme: Theme,
    pub overhang_visualization: (bool, f32),
    pub recent_projects: Vec<PathBuf>,
    pub panels: Option<Tree<Tab>>,
    pub about: bool,
    pub tasks: bool,
    pub default_slice_config: SliceConfig,
    pub slice_preview_mode: SlicePreviewCoordinateSpace,
    pub slice_preview_view: SlicePreviewView,
    pub slice_preview_multisample: u32,

    pub remote_print: RemotePrintConfig,

    pub render_style: RenderStyle,
    pub projection: Projection,
    pub spacenav: SpacenavConfig,
    pub show_normals: bool,
    pub max_buffer_size: u64,
    pub printers: Vec<PrinterProperties>,
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

#[derive(Default, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum SlicePreviewCoordinateSpace {
    ScreenSpace,
    #[default]
    WorldSpace,
}

#[derive(Default, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum SlicePreviewView {
    Screen,
    #[default]
    BuildPlate,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PrinterProperties {
    pub name: Cow<'static, str>,
    pub resolution: Vector2<u32>,
    pub size: Vector3<Milimeters>,
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
            ContentType::Text => "Text",
            ContentType::Json => "JSON",
        }
    }

    pub fn header(&self) -> &str {
        match self {
            ContentType::Text => "text/plain; charset=utf-8",
            ContentType::Json => "application/json",
        }
    }
}

impl SlicePreviewCoordinateSpace {
    pub const ALL: &[Self] = &[Self::ScreenSpace, Self::WorldSpace];

    pub fn name(&self) -> &str {
        match self {
            SlicePreviewCoordinateSpace::ScreenSpace => "Screen Space",
            SlicePreviewCoordinateSpace::WorldSpace => "World Space",
        }
    }
}

impl SlicePreviewView {
    pub const ALL: &[Self] = &[Self::Screen, Self::BuildPlate];

    pub fn name(&self) -> &str {
        match self {
            SlicePreviewView::Screen => "Screen",
            SlicePreviewView::BuildPlate => "Build Plate",
        }
    }
}

impl PrinterProperties {
    pub const fn new(name: &'static str, [rx, ry]: [u32; 2], [sx, sy, sz]: [f32; 3]) -> Self {
        Self {
            name: Cow::Borrowed(name),
            resolution: Vector2::new(rx, ry),
            size: Vector3::new(
                Milimeters::new(sx),
                Milimeters::new(sy),
                Milimeters::new(sz),
            ),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            grid_size: 12.16,
            theme: Theme::Dark,
            overhang_visualization: (false, 30.0),
            default_slice_config: SliceConfig::default(),
            slice_preview_mode: SlicePreviewCoordinateSpace::WorldSpace,
            slice_preview_view: SlicePreviewView::BuildPlate,
            slice_preview_multisample: 8,

            recent_projects: Vec::new(),
            panels: None,
            about: true,
            tasks: false,

            remote_print: Default::default(),

            render_style: RenderStyle::Rendered,
            projection: Projection::Perspective,
            spacenav: Default::default(),
            max_buffer_size: 512 << 20,
            show_normals: false,
            printers: vec![PrinterProperties::new(
                "Custom Printer",
                [11_520, 5_120],
                [218.88, 122.904, 260.0],
            )],
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
