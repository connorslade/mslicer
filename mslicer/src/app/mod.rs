use std::path::PathBuf;

use egui::{Theme, ViewportCommand, Visuals};
use egui_tracing::EventCollector;
use egui_wgpu::RenderState;
use nalgebra::{Vector2, Vector3};
use remote_print::manager::RemotePrintManager;
use tracing::{info, warn};

use crate::{
    app::{
        camera::{Camera, spacenav::SpaceNav},
        config::{Config, printers::selected_printer},
        fps_tracker::FpsTracker,
        history::History,
        slice_operation::SliceOperation,
    },
    project::{Project, model::ModelId},
    render::{Gcx, workspace::model},
    task::{PrinterScan, TaskManager, update_check_if_scheduled},
    ui::{
        drag_and_drop,
        panels::Panels,
        popup::PopupManager,
        state::{RemotePrintConnectStatus, UiState, WorkspaceHover},
    },
    windows,
};

pub mod camera;
pub mod config;
mod fps_tracker;
pub mod history;
mod slice;
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
    pub fn init(
        render_state: RenderState,
        config_dir: PathBuf,
        mut config: Config,
        event_collector: EventCollector,
    ) -> Self {
        let mut spacenav = SpaceNav::unconnected();
        spacenav.try_connect();

        let slice_config = config.default_slice_config.clone();
        let mut this = Self {
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
        };

        update_check_if_scheduled(&mut this);
        if !this.remote_print.is_initialized() && this.config.remote_print.init_at_startup {
            windows::remote_print::initialize(&mut this);

            // Initial scan
            this.state.remote_print_connecting = RemotePrintConnectStatus::Scanning;
            this.tasks.add(PrinterScan::new(
                &this.remote_print,
                this.config.remote_print.broadcast_address,
            ));
        }

        this
    }

    pub fn is_slicing(&self) -> bool {
        is_slicing(&self.slice_operation)
    }

    pub fn gcx(&self) -> Gcx {
        let state = &self.render_state;
        Gcx {
            device: state.device.clone(),
            queue: state.queue.clone(),
            texture: state.target_format,
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
        let projection = self.config.render.projection;
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

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        self.set_title(ctx);

        self.panels.update(ctx.viewport_rect().width());
        self.fps.update();
        self.history
            .set_max_mesh_size(self.config.ui.history_max_mesh_size);
        self.popup().render(ctx);
        self.tasks().poll();

        // todo: only update the visuals if the theme has changed?
        ctx.set_visuals(Visuals {
            collapsing_header_frame: true,
            ..match self.config.ui.theme {
                Theme::Dark => Visuals::dark(),
                Theme::Light => Visuals::light(),
            }
        });

        model::process_previews(self);
        drag_and_drop::update(self, ctx);
        windows::ui(self, ctx);
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // todo: save all surfaces (except slice operation?)
        self.config.ui.panels = Some(self.panels.dock_state.main_surface().clone());
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
