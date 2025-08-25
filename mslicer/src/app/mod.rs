use std::{
    io::{BufRead, Seek},
    mem,
    path::PathBuf,
    sync::Arc,
    thread,
    time::Instant,
};

use clone_macro::clone;
use common::annotations::Annotations;
use const_format::concatcp;
use eframe::Theme;
use egui::{Vec2, Visuals};
use egui_dock::{DockState, NodeIndex, Tree};
use egui_phosphor::regular::CARET_RIGHT;
use egui_tracing::EventCollector;
use egui_wgpu::RenderState;
use nalgebra::Vector2;
use parking_lot::RwLock;
use tracing::{info, warn};

use crate::{
    plugins::{
        anti_alias,
        elephant_foot_fixer::{self},
        PluginManager,
    },
    render::{camera::Camera, preview, rendered_mesh::RenderedMesh},
    ui::{
        drag_and_drop,
        popup::{Popup, PopupIcon, PopupManager},
        state::UiState,
    },
    windows::{self, Tab},
};
use common::config::SliceConfig;
use slicer::{format::FormatSliceFile, slicer::Slicer, Pos};

pub mod config;
pub mod post_processing_operation;
pub mod project;
pub mod remote_print;
pub mod slice_operation;
use config::Config;
use post_processing_operation::PostProcessingOperation;
use remote_print::RemotePrint;
use slice_operation::{SliceOperation, SliceResult};

pub struct App {
    pub render_state: RenderState,
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
    pub post_processing_operation: Option<PostProcessingOperation>,
    pub remote_print: RemotePrint,
    pub config_dir: PathBuf,
}

pub struct FpsTracker {
    last_frame: Instant,
    last_frame_time: f32,
}

impl App {
    pub fn new(
        render_state: RenderState,
        config_dir: PathBuf,
        mut config: Config,
        event_collector: EventCollector,
    ) -> Self {
        let mut dock_state = DockState::new(vec![Tab::Viewport]);
        let surface = dock_state.main_surface_mut();

        if let Some(past_state) = &mut config.panels {
            *surface = mem::take(past_state);
        } else {
            default_dock_layout(surface);
        }

        if surface.find_tab(&Tab::Viewport).is_none() {
            *surface = Tree::new(vec![Tab::Viewport]);
        }

        Self {
            render_state,
            dock_state,
            popup: PopupManager::new(),
            state: UiState {
                event_collector,
                ..Default::default()
            },
            config,
            camera: Camera::default(),
            slice_config: SliceConfig::default(),
            plugin_manager: PluginManager {
                plugins: vec![elephant_foot_fixer::get_plugin(), anti_alias::get_plugin()],
            },
            fps: FpsTracker::new(),
            meshes: Arc::new(RwLock::new(Vec::new())),
            slice_operation: None,
            post_processing_operation: Some(PostProcessingOperation::new(vec![])),
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
                " Import Model or drag and drop a model file into the workspace."
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

            let (min, max) = mesh.bounds();
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

        if let Some(panel) = self.dock_state.find_tab(&Tab::SliceOperation) {
            self.dock_state.set_active_tab(panel);
        } else {
            let window_id = self.dock_state.add_window(vec![Tab::SliceOperation]);
            let window = self.dock_state.get_window_state_mut(window_id).unwrap();
            window.set_size(Vec2::new(700.0, 400.0));
        }

        thread::spawn(clone!(
            [{ self.slice_operation } as slice_operation],
            move || {
                let slice_operation = slice_operation.as_ref().unwrap();
                let slice_result = slicer.slice_format();
                let preview_image = slice_operation.preview_image();
                let file = FormatSliceFile::from_slice_result(preview_image, slice_result);

                let layers = file.info().layers as usize;
                slice_operation.add_result(SliceResult {
                    file,
                    slice_preview_layer: 0,
                    last_preview_layer: 0,
                    preview_offset: Vector2::new(0.0, 0.0),
                    preview_scale: preview_scale.max(1.0).log2(),
                    layer_count: (layers, layers.to_string().len() as u8),
                    annotations: Annotations::default(),
                    show_error_annotations: true,
                    show_warning_annotations: true,
                    show_info_annotations: true,
                    show_debug_annotations: false,
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

        let rendered_mesh = RenderedMesh::from_mesh(model)
            .with_name(name.clone())
            .with_random_color();

        self.meshes.write().push(rendered_mesh);
    }

    pub fn reset_ui(&mut self) {
        self.dock_state = DockState::new(vec![Tab::Viewport]);
        let surface = self.dock_state.main_surface_mut();
        default_dock_layout(surface);
    }

    pub fn show_post_processing(&mut self) {
        if let Some(panel) = self.dock_state.find_tab(&Tab::PostProcessing) {
            self.dock_state.set_active_tab(panel);
        } else {
            let window_id = self.dock_state.add_window(vec![Tab::PostProcessing]);
            let window = self.dock_state.get_window_state_mut(window_id).unwrap();
            window.set_size(Vec2::new(700.0, 400.0));
        }
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
        preview::process_previews(app);
        drag_and_drop::update(self, ctx);
        windows::ui(self, ctx);
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.config.panels = Some(self.dock_state.main_surface().clone());
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

fn default_dock_layout(surface: &mut Tree<Tab>) {
    surface.split_right(NodeIndex::root(), 0.7, vec![Tab::About]);
    let [_old_node, new_node] = surface.split_left(NodeIndex::root(), 0.2, vec![Tab::Models]);
    let [_old_node, new_node] =
        surface.split_below(new_node, 0.5, vec![Tab::SliceConfig, Tab::Supports]);
    surface.split_below(new_node, 0.5, vec![Tab::Workspace, Tab::RemotePrint]);
}
