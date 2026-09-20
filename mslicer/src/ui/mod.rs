use std::ops::{Deref, DerefMut};

use const_format::concatcp;
use egui::{Theme, ViewportCommand, Visuals};
use egui_phosphor::regular::CARET_RIGHT;
use egui_tracing::EventCollector;
use egui_wgpu::RenderState;
use mslicer_core::{
    config::ui::Tab,
    core::{Core, SLICE_PREVIEW_SIZE},
};
use nalgebra::Vector3;

use crate::{
    camera::{Camera, spacenav::SpaceNav},
    render::{Gcx, workspace::model},
    task::{PrinterScan, TaskManager, update_check_if_scheduled},
    ui::{
        fps_tracker::FpsTracker,
        panels::Panels,
        popup::{Popup, PopupIcon, PopupManager},
        selected::selected_printer,
        state::{RemotePrintConnectStatus, UiState, WorkspaceHover},
    },
    windows,
};

pub mod components;
pub mod drag_and_drop;
pub mod fps_tracker;
pub mod panels;
pub mod popup;
pub mod selected;
pub mod shortcuts;
pub mod state;

// a fresh start <3
pub struct App {
    pub core: Core,
    pub render_state: RenderState,

    pub camera: Camera,
    pub spacenav: SpaceNav,

    pub fps: FpsTracker,
    pub panels: Panels,
    pub popup: PopupManager,
    pub tasks: TaskManager,
    pub state: UiState,
}

impl App {
    pub fn init(mut core: Core, render_state: RenderState, collector: EventCollector) -> Self {
        let mut spacenav = SpaceNav::unconnected();
        spacenav.try_connect();

        let mut this = Self {
            panels: Panels::new(&mut core.config),
            state: UiState {
                selected_printer: selected_printer(&core.config, &core.project.slice_config),
                event_collector: collector,
                ..Default::default()
            },
            fps: FpsTracker::new(),
            popup: PopupManager::default(),
            tasks: TaskManager::new(),

            core,
            render_state,

            camera: Camera::default(),
            spacenav,
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

    pub fn slice(&mut self) {
        if self.core.slice() {
            self.panels.focus_tab(Tab::Sliced, SLICE_PREVIEW_SIZE);
        } else {
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
        }
    }

    pub fn hovered_ray(&self) -> Option<(Vector3<f32>, Vector3<f32>)> {
        let projection = self.config.render.projection;
        let WorkspaceHover { aspect, uv, .. } = &self.state.workspace;

        self.state
            .workspace
            .hovered()
            .then(|| self.camera.hovered_ray(projection, *aspect, *uv))
    }

    pub fn gcx(&self) -> Gcx {
        let state = &self.render_state;
        Gcx {
            device: state.device.clone(),
            queue: state.queue.clone(),
            texture: state.target_format,
        }
    }

    fn set_title(&mut self, ctx: &egui::Context) {
        let stem = self.core.project.path.as_ref().and_then(|x| x.file_stem());
        let title = if let Some(stem) = stem {
            format!("mslicer — {}", stem.to_string_lossy())
        } else {
            "mslicer".into()
        };
        ctx.send_viewport_cmd(ViewportCommand::Title(title));
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        self.set_title(ctx);

        self.core.update();
        self.panels.update(ctx.viewport_rect().width());
        self.fps.update();
        self.popup().render(ctx);
        self.tasks().poll();

        // todo: only update the visuals if the theme has changed?
        ctx.set_visuals(Visuals {
            collapsing_header_frame: true,
            ..match self.core.config.ui.theme {
                Theme::Dark => Visuals::dark(),
                Theme::Light => Visuals::light(),
            }
        });

        model::process_previews(self);
        drag_and_drop::update(self, ctx);
        windows::ui(self, ctx);
    }
}

impl Deref for App {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        &self.core
    }
}

impl DerefMut for App {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.core
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.core.config.ui.panels = Some(self.panels.dock_state.main_surface().clone());
    }
}
