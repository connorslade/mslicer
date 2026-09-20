use std::path::PathBuf;

use egui::{Theme, ViewportCommand, Visuals};
use egui_tracing::EventCollector;
use egui_wgpu::RenderState;
use nalgebra::{Vector2, Vector3};
use remote_print::manager::RemotePrintManager;
use tracing::{info, warn};

use crate::{
    core::{
        config::{Config, printers::selected_printer},
        history::History,
        project::Project,
        slice_operation::SliceOperation,
        state::{RemotePrintConnectStatus, UiState, WorkspaceHover},
    },
    interface::{
        self,
        panels::{self, Panels},
        popup::PopupManager,
    },
    render::{
        Gcx,
        camera::{Camera, spacenav::SpaceNav},
        workspace::model,
    },
    task::{PrinterScan, TaskManager, update_check_if_scheduled},
    util::fps_tracker::FpsTracker,
};

pub const SLICE_PREVIEW_SIZE: Vector2<f32> = Vector2::new(700.0, 400.0);

pub struct App {
    // Core
    pub config_dir: PathBuf,
    pub config: Config,
    pub project: Project,
    pub history: History,
    pub remote_print: RemotePrintManager,
    pub slice_operation: Option<SliceOperation>,

    // Rendering
    pub render_state: RenderState,
    pub panels: Panels,
    pub fps: FpsTracker,
    pub camera: Camera,
    pub spacenav: SpaceNav,

    // Interface
    pub state: UiState,
    pub popup: PopupManager,
    pub tasks: TaskManager,
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
            panels::remote_print::initialize(&mut this);

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
        interface::ui(self, ctx);
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
