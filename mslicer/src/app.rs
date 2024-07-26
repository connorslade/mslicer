use std::{sync::Arc, thread, time::Instant};

use clone_macro::clone;
use eframe::Theme;
use egui_dock::{DockState, NodeIndex};
use egui_modal::{DialogBuilder, Icon, Modal};
use egui_tracing::EventCollector;
use image::imageops::FilterType;
use nalgebra::{Vector2, Vector3};
use parking_lot::RwLock;
use slicer::{slicer::Slicer, Pos};
use tracing::info;

use crate::{
    remote_print::RemotePrint,
    render::{camera::Camera, pipelines::model::RenderStyle, rendered_mesh::RenderedMesh},
    slice_operation::{SliceOperation, SliceResult},
    windows::{self, Tab},
};
use common::config::{ExposureConfig, SliceConfig};
use goo_format::{File as GooFile, LayerEncoder, PreviewImage};

pub struct App {
    pub dock_state: DockState<Tab>,
    pub state: UiState,
    modal: Option<Modal>,

    pub camera: Camera,
    pub slice_config: SliceConfig,
    pub meshes: Arc<RwLock<Vec<RenderedMesh>>>,
    pub slice_operation: Option<SliceOperation>,
    pub remote_print: RemotePrint,

    pub render_style: RenderStyle,
    pub grid_size: f32,
    pub fps: FpsTracker,
    pub theme: Theme,
    pub alert_print_completion: bool,
    pub init_remote_print_at_startup: bool,
}

#[derive(Default)]
pub struct UiState {
    pub event_collector: EventCollector,
    pub working_address: String,
    pub send_print_completion: bool,
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
        let [_old_node, new_node] = surface.split_below(new_node, 0.5, vec![Tab::SliceConfig]);
        surface.split_below(new_node, 0.5, vec![Tab::Workspace, Tab::RemotePrint]);

        Self {
            dock_state,
            modal: None,
            state: UiState {
                event_collector,
                ..Default::default()
            },

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
                    exposure_time: 50.0,
                    ..Default::default()
                },
                first_layers: 10,
            },

            fps: FpsTracker::new(),
            meshes: Arc::new(RwLock::new(Vec::new())),
            render_style: RenderStyle::Rended,
            theme: Theme::Dark,
            grid_size: 12.16,
            slice_operation: None,
            remote_print: RemotePrint::uninitialized(),
            alert_print_completion: false,
            init_remote_print_at_startup: false,
        }
    }

    pub fn slice(&mut self) {
        if self.meshes.read().is_empty() {
            const NO_MODELS_ERROR: &str = "There are no models to slice. Add one by going to File → Open Model or drag and drop a model file into the workspace.";
            self.dialog_builder()
                .with_title("Slicing Error")
                .with_body(NO_MODELS_ERROR)
                .with_icon(Icon::Error)
                .open();
            return;
        }

        info!("Starting slicing operation");

        let slice_config = self.slice_config.clone();
        let mut meshes = Vec::new();
        let mut preview_scale = f32::MAX;

        let mm_to_px = Pos::new(
            self.slice_config.platform_resolution.x as f32 / self.slice_config.platform_size.x,
            self.slice_config.platform_resolution.y as f32 / self.slice_config.platform_size.y,
            1.0,
        );

        for mesh in self.meshes.read().iter().cloned() {
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
                        pos.z - self.slice_config.slice_height,
                    ),
            );

            mesh.update_transformation_matrix();

            meshes.push(mesh);
        }

        let slicer = Slicer::new(slice_config, meshes);
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

                slice_operation.add_result(SliceResult {
                    goo,
                    slice_preview_layer: 0,
                    last_preview_layer: 0,
                    preview_offset: Vector2::new(0.0, 0.0),
                    preview_scale: preview_scale.max(1.0),
                });
            }
        ));
    }

    pub fn dialog_builder(&mut self) -> DialogBuilder {
        self.modal.as_mut().unwrap().dialog()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        self.fps.update();

        match &mut self.modal {
            Some(modal) => modal.show_dialog(),
            None => self.modal = Some(Modal::new(ctx, "modal")),
        }

        windows::ui(self, ctx);
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
