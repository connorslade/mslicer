use std::{path::PathBuf, sync::Arc, thread};

use clone_macro::clone;
use const_format::concatcp;
use egui::{Theme, ViewportCommand, Visuals};
use egui_phosphor::regular::CARET_RIGHT;
use egui_tracing::EventCollector;
use egui_wgpu::RenderState;
use nalgebra::{Vector2, Vector3};
use remote_print::manager::RemotePrintManager;
use tracing::{info, warn};

use crate::{
    app::{
        config::{Config, DEFAULT_PRINTERS},
        fps_tracker::FpsTracker,
        history::History,
        slice_operation::SliceOperation,
    },
    project::{Project, model::ModelId},
    render::{
        Gcx,
        camera::{Camera, spacenav::SpaceNav},
        preview,
    },
    task::TaskManager,
    ui::{
        drag_and_drop,
        panels::Panels,
        popup::{Popup, PopupIcon, PopupManager},
        state::{SelectedPrinter, UiState, WorkspaceHover},
    },
    windows::{self, Tab},
};
use common::{
    progress::CombinedProgress,
    slice::{SliceConfig, SliceMode},
    units::Milimeter,
};
use slicer::slicer::{Slicer, SlicerModel};

pub mod config;
mod fps_tracker;
pub mod history;
pub mod slice_operation;

pub const SLICE_PREVIEW_SIZE: Vector2<f32> = Vector2::new(700.0, 400.0);

pub struct App {
    pub render_state: RenderState,
    pub panels: Panels,
    pub fps: FpsTracker,
    pub config_dir: PathBuf,

    pub popup: PopupManager,
    pub tasks: TaskManager,
    pub remote_print: RemotePrintManager,
    pub slice_operation: Option<SliceOperation>,

    pub camera: Camera,
    pub spacenav: SpaceNav,
    pub state: UiState,
    pub history: History,

    pub config: Config,
    pub project: Project,
}

impl App {
    pub fn new(
        render_state: RenderState,
        config_dir: PathBuf,
        mut config: Config,
        event_collector: EventCollector,
    ) -> Self {
        let mut spacenav = SpaceNav::unconnected();
        spacenav.try_connect();

        let slice_config = config.default_slice_config.clone();
        Self {
            render_state,
            panels: Panels::new(&mut config),
            fps: FpsTracker::new(),
            config_dir,
            popup: PopupManager::default(),
            tasks: TaskManager::new(),
            remote_print: RemotePrintManager::default(),
            slice_operation: None,
            camera: Camera::default(),
            spacenav,
            state: UiState {
                event_collector,
                selected_printer: selected_printer(&config, &slice_config),
                ..Default::default()
            },
            history: History::default(),
            config,
            project: Project {
                slice_config,
                ..Default::default()
            },
        }
    }

    pub fn is_slicing(&self) -> bool {
        is_slicing(&self.slice_operation)
    }

    pub fn gcx(&self) -> Gcx {
        Gcx {
            device: self.render_state.device.clone(),
            queue: self.render_state.queue.clone(),
        }
    }

    pub fn set_title(&mut self, ctx: &egui::Context) {
        let title = if let Some(stem) = self.project.path.as_ref().and_then(|x| x.file_stem()) {
            format!("mslicer — {}", stem.to_string_lossy())
        } else {
            "mslicer".into()
        };
        ctx.send_viewport_cmd(ViewportCommand::Title(title));
    }

    pub fn hovered_ray(&self) -> Option<(Vector3<f32>, Vector3<f32>)> {
        let projection = self.config.projection;
        let workspace @ WorkspaceHover { aspect, uv, .. } = &self.state.workspace;

        workspace
            .hovered()
            .then(|| self.camera.hovered_ray(projection, *aspect, *uv))
    }

    pub fn hovered_model(&self) -> Option<ModelId> {
        let (pos, dir) = self.hovered_ray()?;
        let mut min = (f32::MAX, ModelId::default());

        for model in self.project.models.iter() {
            if !model.hidden
                && let Some(bvh) = &model.bvh
                && let Some(hit) = bvh.intersect_ray(&model.mesh, pos, dir)
                && hit.t < min.0
            {
                min = (hit.t, model.id);
            }
        }

        (min.0 != f32::MAX).then_some(min.1)
    }
}

impl App {
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
        let slice_height = slice_config.slice_height.get::<Milimeter>();
        let platform_size = (slice_config.platform_size.xy()).map(|x| x.get::<Milimeter>());

        let platform = slice_config.platform_resolution.cast::<f32>();
        let mm_to_px = platform.component_div(&platform_size).push(1.0);

        // Transform models from world-space to platform-space
        let mut out = Vec::new();
        for model in meshes.into_iter() {
            let (mut mesh, exposure) = (model.mesh, model.exposure);

            let offset = (platform / 2.0).push(-slice_height / 2.0);
            mesh.set_scale_unchecked(mesh.scale().component_mul(&mm_to_px));
            mesh.set_position_unchecked(mesh.position().component_mul(&mm_to_px) + offset);
            mesh.update_transformation_matrix();

            out.push(SlicerModel { mesh, exposure });
        }

        let slicer = Slicer::new(slice_config, out);
        let post_process = CombinedProgress::new();
        let slice_operation = SliceOperation::new(slicer.progress(), post_process.clone());
        self.slice_operation.replace(slice_operation);
        self.panels.focus_tab(Tab::SlicePreview, SLICE_PREVIEW_SIZE);

        thread::spawn(clone!(
            [
                { self.slice_operation } as slice_operation,
                { self.project.post_processing } as post_processing
            ],
            move || {
                let slice_operation = slice_operation.as_ref().unwrap();

                match slicer.slice_config.mode {
                    SliceMode::Raster => {
                        let mut layers = slicer.slice_raster();
                        post_processing.process(&slicer.slice_config, &mut layers, post_process);
                        slice_operation.add_raster_result(slicer.slice_config, layers);
                    }
                    SliceMode::Vector => {
                        let layers = slicer.slice_vector();
                        slice_operation.add_vector_result(slicer.slice_config, Arc::new(layers));
                    }
                }
            }
        ));
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        self.set_title(ctx);

        self.panels.update(ctx.viewport_rect().width());
        self.fps.update();
        self.popup().render(ctx);
        self.tasks().poll();

        // only update the visuals if the theme has changed
        ctx.set_visuals(Visuals {
            collapsing_header_frame: true,
            ..match self.config.theme {
                Theme::Dark => Visuals::dark(),
                Theme::Light => Visuals::light(),
            }
        });

        // self.remote_print().tick(); TODO(remote-print)
        preview::process_previews(self);
        drag_and_drop::update(self, ctx);
        windows::ui(self, ctx);
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // todo: save all surfaces (except slice operation?)
        self.config.panels = Some(self.panels.dock_state.main_surface().clone());
        if let Err(err) = self.config.save(&self.config_dir) {
            warn!("Failed to save config: {}", err);
        } else {
            info!("Successfully saved config");
        }
    }
}

pub fn is_slicing(slice_operation: &Option<SliceOperation>) -> bool {
    slice_operation
        .as_ref()
        .map(|x| !x.progress.complete())
        .unwrap_or_default()
}

fn selected_printer(config: &Config, slice_config: &SliceConfig) -> SelectedPrinter {
    for (i, printer) in config.printers.iter().enumerate() {
        if printer.resolution == slice_config.platform_resolution
            && printer.size == slice_config.platform_size
        {
            return SelectedPrinter::Custom(i);
        }
    }

    for (i, brand) in DEFAULT_PRINTERS.iter().enumerate() {
        for (j, printer) in brand.1.iter().enumerate() {
            if printer.resolution == slice_config.platform_resolution
                && printer.size == slice_config.platform_size
            {
                return SelectedPrinter::Preset(i, j);
            }
        }
    }

    SelectedPrinter::Project
}
