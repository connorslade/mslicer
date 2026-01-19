use std::{mem, path::PathBuf, thread, time::Instant};

use clone_macro::clone;
use const_format::concatcp;
use egui::{Theme, Vec2, Visuals};
use egui_dock::{DockState, NodeIndex, Tree};
use egui_phosphor::regular::CARET_RIGHT;
use egui_tracing::EventCollector;
use egui_wgpu::RenderState;
use nalgebra::Vector2;
use tracing::{info, warn};

use crate::{
    app::{
        config::Config,
        project::Project,
        remote_print::RemotePrint,
        slice_operation::{SliceOperation, SliceResult},
        task::TaskManager,
    },
    render::{camera::Camera, preview},
    ui::{
        drag_and_drop,
        popup::{Popup, PopupIcon, PopupManager},
        state::UiState,
    },
    windows::{self, Tab},
};
use common::config::SliceConfig;
use slicer::{format::FormatSliceFile, slicer::Slicer};

pub mod config;
pub mod project;
pub mod remote_print;
pub mod slice_operation;
pub mod task;

pub struct App {
    pub render_state: RenderState,
    pub dock_state: DockState<Tab>,
    pub fps: FpsTracker,
    pub config_dir: PathBuf,

    pub popup: PopupManager,
    pub tasks: TaskManager,
    pub remote_print: RemotePrint,
    pub slice_operation: Option<SliceOperation>,

    pub camera: Camera,
    pub state: UiState,
    pub config: Config,
    pub project: Project,
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

        match surface.find_tab(&Tab::Viewport) {
            Some((ni, ti)) => surface.set_active_tab(ni, ti),
            None => *surface = Tree::new(vec![Tab::Viewport]),
        }

        let slice_config = SliceConfig::default();
        let selected_printer = (config.printers.iter())
            .position(|x| {
                x.resolution == slice_config.platform_resolution
                    && x.size == slice_config.platform_size
            })
            .map(|x| x + 1)
            .unwrap_or_default();

        Self {
            render_state,
            dock_state,
            fps: FpsTracker::new(),
            config_dir,
            popup: PopupManager::default(),
            tasks: TaskManager::default(),
            remote_print: RemotePrint::uninitialized(),
            slice_operation: None,
            camera: Camera::default(),
            state: UiState {
                event_collector,
                selected_printer,
                ..Default::default()
            },
            config,
            project: Project {
                slice_config,
                ..Default::default()
            },
        }
    }

    pub fn slice(&mut self) {
        let meshes = (self.project.models.iter())
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

        let slice_config = self.project.slice_config.clone();
        let mut out = Vec::new();
        let mut preview_scale = f32::MAX;

        let platform = slice_config.platform_resolution.cast::<f32>();
        let mm_to_px = platform
            .component_div(&slice_config.platform_size.xy())
            .push(1.0);

        for mesh in meshes.into_iter() {
            let mut mesh = mesh.mesh;
            mesh.set_scale_unchecked(mesh.scale().component_mul(&mm_to_px));

            let (min, max) = mesh.bounds();
            preview_scale = preview_scale
                .min(slice_config.platform_size.x / (max.x - min.x))
                .min(slice_config.platform_size.y / (max.y - min.y));

            let pos = mesh.position();
            let offset = (platform / 2.0).push(-slice_config.slice_height);
            mesh.set_position_unchecked(pos.component_mul(&mm_to_px) + offset);
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
                });
            }
        ));
    }

    pub fn reset_ui(&mut self) {
        self.dock_state = DockState::new(vec![Tab::Viewport]);
        let surface = self.dock_state.main_surface_mut();
        default_dock_layout(surface);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        self.fps.update();

        // todo: probably dont do this
        let app = unsafe { &mut *(self as *mut _) };
        self.popup.render(app, ctx);
        self.tasks.render(app, ctx);

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
