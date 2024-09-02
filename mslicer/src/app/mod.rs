use std::{
    fs::File,
    io::{BufRead, Seek},
    path::{Path, PathBuf},
    sync::Arc,
    thread,
    time::Instant,
};

use anyhow::Result;
use clone_macro::clone;
use const_format::concatcp;
use eframe::Theme;
use egui::Visuals;
use egui_dock::{DockState, NodeIndex};
use egui_phosphor::regular::CARET_RIGHT;
use egui_tracing::EventCollector;
use image::imageops::FilterType;
use nalgebra::{Vector2, Vector3};
use parking_lot::RwLock;
use slicer::{slicer::Slicer, Pos};
use tracing::{info, warn};

use crate::{
    plugins::{
        elephant_foot_fixer::{self},
        PluginManager,
    },
    render::{camera::Camera, rendered_mesh::RenderedMesh},
    ui::{
        drag_and_drop,
        popup::{Popup, PopupIcon, PopupManager},
        state::UiState,
    },
    windows::{self, Tab},
};
use common::config::{ExposureConfig, SliceConfig};
use goo_format::{File as GooFile, LayerEncoder, PreviewImage};

pub mod config;
pub mod project;
pub mod remote_print;
pub mod slice_operation;
use config::Config;
use remote_print::RemotePrint;
use slice_operation::{SliceOperation, SliceResult};

use project::{BorrowedProject, OwnedProject};

pub struct App {
    // todo: dock state in ui_state?
    pub dock_state: DockState<Tab>,
    pub fps: FpsTracker,
    pub popup: PopupManager,

    pub state: UiState,
    pub config: Config,
    pub slice_config: SliceConfig,
    pub plugin_manager: PluginManager,

    pub camera: Camera,
    pub meshes: Arc<RwLock<Vec<RenderedMesh>>>,
    pub slice_operation: Option<SliceOperation>,
    pub remote_print: RemotePrint,
    pub config_dir: PathBuf,
}

pub struct FpsTracker {
    last_frame: Instant,
    last_frame_time: f32,
}

impl App {
    pub fn new(event_collector: EventCollector) -> Self {
        let mut dock_state = DockState::new(vec![Tab::Viewport, Tab::Logs]);
        let surface = dock_state.main_surface_mut();
        let [_old_node, new_node] = surface.split_left(NodeIndex::root(), 0.20, vec![Tab::Models]);
        let [_old_node, new_node] =
            surface.split_below(new_node, 0.5, vec![Tab::SliceConfig, Tab::Supports]);
        surface.split_below(new_node, 0.5, vec![Tab::Workspace, Tab::RemotePrint]);

        let config_dir = dirs::config_dir().unwrap().join("mslicer");
        let config = match Config::load(&config_dir) {
            Ok(config) => config,
            Err(err) => {
                warn!("Failed to load config, using defaults: {}", err);
                Config::default()
            }
        };

        Self {
            dock_state,
            popup: PopupManager::new(),
            state: UiState {
                event_collector,
                ..Default::default()
            },
            config,
            camera: Camera::default(),
            slice_config: SliceConfig {
                platform_resolution: Vector2::new(11_520, 5_120),
                platform_size: Vector3::new(218.88, 122.904, 260.0),
                slice_height: 0.05,
                exposure_config: ExposureConfig {
                    exposure_time: 3.0,
                    ..Default::default()
                },
                first_exposure_config: ExposureConfig {
                    exposure_time: 30.0,
                    ..Default::default()
                },
                first_layers: 3,
                transition_layers: 10,
            },
            plugin_manager: PluginManager {
                plugins: vec![elephant_foot_fixer::get_plugin()],
            },
            fps: FpsTracker::new(),
            meshes: Arc::new(RwLock::new(Vec::new())),
            slice_operation: None,
            remote_print: RemotePrint::uninitialized(),
            config_dir,
        }
    }

    pub fn slice(&mut self) {
        let meshes = self
            .meshes
            .read()
            .iter()
            .filter(|x| !x.hidden)
            .cloned()
            .collect::<Vec<_>>();

        if meshes.is_empty() {
            const NO_MODELS_ERROR: &str = concatcp!(
                "There are no models to slice. Add one by going to File ",
                CARET_RIGHT,
                " Open Model or drag and drop a model file into the workspace."
            );
            self.popup.open(Popup::simple(
                "Slicing Error",
                PopupIcon::Error,
                NO_MODELS_ERROR,
            ));
            return;
        }

        info!("Starting slicing operation");

        let slice_config = self.slice_config.clone();
        let mut out = Vec::new();
        let mut preview_scale = f32::MAX;

        let mm_to_px = Pos::new(
            self.slice_config.platform_resolution.x as f32 / self.slice_config.platform_size.x,
            self.slice_config.platform_resolution.y as f32 / self.slice_config.platform_size.y,
            1.0,
        );

        for mesh in meshes.into_iter() {
            let mut mesh = mesh.mesh;

            mesh.set_scale_unchecked(mesh.scale().component_mul(&mm_to_px));

            let (min, max) = mesh.minmax_point();
            preview_scale = preview_scale
                .min(self.slice_config.platform_size.x / (max.x - min.x))
                .min(self.slice_config.platform_size.y / (max.y - min.y));

            let pos = mesh.position();
            mesh.set_position_unchecked(
                pos.component_mul(&mm_to_px)
                    + Pos::new(
                        self.slice_config.platform_resolution.x as f32 / 2.0,
                        self.slice_config.platform_resolution.y as f32 / 2.0,
                        -self.slice_config.slice_height,
                    ),
            );

            mesh.update_transformation_matrix();

            out.push(mesh);
        }

        let slicer = Slicer::new(slice_config, out);
        self.slice_operation
            .replace(SliceOperation::new(slicer.progress()));

        thread::spawn(clone!(
            [{ self.slice_operation } as slice_operation],
            move || {
                let slice_operation = slice_operation.as_ref().unwrap();
                let mut goo = GooFile::from_slice_result(slicer.slice::<LayerEncoder>());

                {
                    let preview_image = slice_operation.preview_image();
                    goo.header.big_preview =
                        PreviewImage::from_image_scaled(&preview_image, FilterType::Nearest);
                    goo.header.small_preview =
                        PreviewImage::from_image_scaled(&preview_image, FilterType::Nearest);
                }

                let layers = goo.layers.len();
                slice_operation.add_result(SliceResult {
                    goo,
                    slice_preview_layer: 0,
                    last_preview_layer: 0,
                    preview_offset: Vector2::new(0.0, 0.0),
                    preview_scale: preview_scale.max(1.0).log2(),
                    layer_count: (layers, layers.to_string().len() as u8),
                });
            }
        ));
    }

    pub fn load_mesh<T: BufRead + Seek>(&mut self, buf: &mut T, format: &str, name: String) {
        let model = match slicer::mesh::load_mesh(buf, format) {
            Ok(model) => model,
            Err(err) => {
                self.popup.open(Popup::simple(
                    "Import Error",
                    PopupIcon::Error,
                    format!("Failed to import model.\n{err}"),
                ));
                return;
            }
        };
        info!("Loaded model `{name}` with {} faces", model.face_count());

        self.meshes.write().push(
            RenderedMesh::from_mesh(model)
                .with_name(name)
                .with_random_color(),
        );
    }

    pub fn save_project(&self, path: &Path) -> Result<()> {
        let meshes = self.meshes.read();
        let project = BorrowedProject::new(&meshes, &self.slice_config);

        let mut file = File::create(path)?;
        project.serialize(&mut file)?;
        Ok(())
    }

    pub fn load_project(&mut self, path: &Path) -> Result<()> {
        let mut file = File::open(path)?;
        let project = OwnedProject::deserialize(&mut file)?;

        self.config.recent_projects.push(path.to_path_buf());

        project.apply(self);
        Ok(())
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        self.fps.update();

        // todo: probably dont do this
        let app = unsafe { &mut *(self as *mut _) };
        self.popup.render(app, ctx);
        // only update the visuals if the theme has changed

        match self.config.theme {
            Theme::Dark => ctx.set_visuals(Visuals::dark()),
            Theme::Light => ctx.set_visuals(Visuals::light()),
        }

        if let Some(operation) = &mut self.slice_operation {
            operation.post_process_if_needed(app);
        }

        self.remote_print.tick(app);
        drag_and_drop::update(self, ctx);
        windows::ui(self, ctx);
    }
}

impl Drop for App {
    fn drop(&mut self) {
        if let Err(err) = self.config.save(&self.config_dir) {
            warn!("Failed to save config: {}", err);
            return;
        }
        info!("Successfully saved config");
    }
}

impl FpsTracker {
    fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            last_frame_time: 0.0,
        }
    }

    fn update(&mut self) {
        let now = Instant::now();
        let elapsed = now - self.last_frame;
        self.last_frame_time = elapsed.as_secs_f32();
        self.last_frame = now;
    }

    pub fn frame_time(&self) -> f32 {
        self.last_frame_time
    }
}
