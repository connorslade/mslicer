use std::{sync::Arc, thread, time::Instant};

use clone_macro::clone;
use egui::{CentralPanel, Frame, Sense};
use egui_wgpu::Callback;
use image::{imageops::FilterType, RgbaImage};
use nalgebra::{Vector2, Vector3};
use parking_lot::{lock_api::MutexGuard, Condvar, MappedMutexGuard, Mutex, RawMutex, RwLock};
use slicer::{
    slicer::{Progress as SliceProgress, Slicer},
    Pos,
};
use tracing::info;

use crate::{
    render::{
        camera::Camera, pipelines::model::RenderStyle, rendered_mesh::RenderedMesh,
        workspace::WorkspaceRenderCallback,
    },
    windows::{self, Windows},
};
use common::config::{ExposureConfig, SliceConfig};
use goo_format::{File as GooFile, LayerEncoder, PreviewImage};

pub struct App {
    pub camera: Camera,
    pub slice_config: SliceConfig,
    pub meshes: Arc<RwLock<Vec<RenderedMesh>>>,

    pub slice_operation: Option<SliceOperation>,

    pub render_style: RenderStyle,
    pub grid_size: f32,
    pub fps: FpsTracker,
    pub windows: Windows,
}

pub struct FpsTracker {
    last_frame: Instant,
    last_frame_time: f32,
}

pub struct SliceResult {
    pub goo: GooFile,

    pub slice_preview_layer: usize,
    pub last_preview_layer: usize,
    pub preview_offset: Vector2<f32>,
    pub preview_scale: f32,
}

// todo: Arc<SliceOperationInner>?
#[derive(Clone)]
pub struct SliceOperation {
    pub progress: SliceProgress,
    pub result: Arc<Mutex<Option<SliceResult>>>,

    pub preview_image: Arc<Mutex<Option<RgbaImage>>>,
    preview_condvar: Arc<Condvar>,
}

impl SliceOperation {
    pub fn new(progress: SliceProgress) -> Self {
        Self {
            progress,
            result: Arc::new(Mutex::new(None)),
            preview_image: Arc::new(Mutex::new(None)),
            preview_condvar: Arc::new(Condvar::new()),
        }
    }

    pub fn needs_preview_image(&self) -> bool {
        self.preview_image.lock().is_none()
    }

    pub fn add_preview_image(&self, image: RgbaImage) {
        self.preview_image.lock().replace(image);
        self.preview_condvar.notify_all();
    }

    pub fn preview_image(&self) -> MappedMutexGuard<'_, RgbaImage> {
        let mut preview_image = self.preview_image.lock();
        while preview_image.is_none() {
            self.preview_condvar.wait(&mut preview_image);
        }

        MutexGuard::map(preview_image, |image| image.as_mut().unwrap())
    }

    pub fn add_result(&self, result: SliceResult) {
        self.result.lock().replace(result);
    }

    pub fn result(&self) -> MutexGuard<RawMutex, Option<SliceResult>> {
        self.result.lock()
    }
}

impl App {
    pub fn slice(&mut self) {
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
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
        self.fps.update();

        windows::ui(self, ctx, frame);

        CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::drag());
                self.camera.handle_movement(&response, ui);

                let callback = Callback::new_paint_callback(
                    rect,
                    WorkspaceRenderCallback {
                        camera: self.camera.clone(),
                        transform: self
                            .camera
                            .view_projection_matrix(rect.width() / rect.height()),

                        bed_size: self.slice_config.platform_size,
                        grid_size: self.grid_size,

                        is_moving: response.dragged(),
                        slice_operation: self.slice_operation.clone(),

                        models: self.meshes.clone(),
                        render_style: self.render_style,
                    },
                );
                ui.painter().add(callback);
            });
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

impl Default for App {
    fn default() -> Self {
        Self {
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
            windows: Windows::default(),
            render_style: RenderStyle::Normals,
            grid_size: 12.16,

            slice_operation: None,
        }
    }
}
