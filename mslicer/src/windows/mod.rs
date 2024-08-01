use eframe::Theme;
use egui::{CentralPanel, Color32, Context, Frame, Id, Sense, Ui, WidgetText};
use egui_dock::{DockArea, NodeIndex, SurfaceIndex, TabViewer};
use egui_wgpu::Callback;

use crate::{app::App, render::workspace::WorkspaceRenderCallback};

mod about;
mod logs;
mod models;
mod remote_print;
mod slice_config;
mod slice_operation;
mod stats;
mod supports;
mod top_bar;
mod workspace;

struct Tabs<'a> {
    app: &'a mut App,
    ctx: &'a Context,
}

#[derive(Hash, PartialEq, Eq)]
pub enum Tab {
    About,
    Logs,
    Models,
    RemotePrint,
    SliceConfig,
    Stats,
    Supports,
    Viewport,
    Workspace,
}

impl Tab {
    pub fn name(&self) -> &'static str {
        match self {
            Tab::About => "About",
            Tab::Logs => "Logs",
            Tab::Models => "Models",
            Tab::RemotePrint => "Remote Print",
            Tab::SliceConfig => "Slice Config",
            Tab::Stats => "Stats",
            Tab::Supports => "Supports",
            Tab::Viewport => "Viewport",
            Tab::Workspace => "Workspace",
        }
    }
}

impl<'a> TabViewer for Tabs<'a> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.name().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::About => about::ui(self.app, ui, self.ctx),
            Tab::Logs => logs::ui(self.app, ui, self.ctx),
            Tab::Models => models::ui(self.app, ui, self.ctx),
            Tab::RemotePrint => remote_print::ui(self.app, ui, self.ctx),
            Tab::SliceConfig => slice_config::ui(self.app, ui, self.ctx),
            Tab::Stats => stats::ui(self.app, ui, self.ctx),
            Tab::Supports => supports::ui(self.app, ui, self.ctx),
            Tab::Viewport => viewport(self.app, ui, self.ctx),
            Tab::Workspace => workspace::ui(self.app, ui, self.ctx),
        }
    }

    fn add_popup(&mut self, ui: &mut Ui, _surface: SurfaceIndex, _node: NodeIndex) {
        ui.set_min_width(120.0);
        ui.style_mut().visuals.button_frame = false;

        for tab in [
            Tab::About,
            Tab::Logs,
            Tab::Models,
            Tab::RemotePrint,
            Tab::SliceConfig,
            Tab::Stats,
            Tab::Supports,
            Tab::Workspace,
        ] {
            let already_open = self.app.dock_state.find_tab(&tab).is_some();
            ui.add_enabled_ui(!already_open, |ui| {
                ui.button(tab.name())
                    .clicked()
                    .then(|| self.app.dock_state.add_window(vec![tab]));
            });
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        *tab != Tab::Viewport
    }

    fn allowed_in_windows(&self, tab: &mut Self::Tab) -> bool {
        *tab != Tab::Viewport
    }

    fn id(&mut self, tab: &mut Self::Tab) -> Id {
        Id::new(tab)
    }

    fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] {
        [false, true]
    }
}

pub fn ui(app: &mut App, ctx: &Context) {
    top_bar::ui(app, ctx);
    slice_operation::ui(app, ctx);

    CentralPanel::default()
        .frame(Frame::none())
        .show(ctx, |ui| {
            // i am once again too tired to deal with this
            let dock_state = unsafe { &mut *(&mut app.dock_state as *mut _) };
            DockArea::new(dock_state)
                .show_add_buttons(true)
                .show_add_popup(true)
                .show_inside(ui, &mut Tabs { app, ctx });
        });
}

fn viewport(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::drag());
    app.camera.handle_movement(&response, ui);

    let color = match app.config.theme {
        Theme::Dark => Color32::from_rgb(9, 9, 9),
        Theme::Light => Color32::from_rgb(255, 255, 255),
    };
    ui.painter().rect_filled(rect, 0.0, color);

    let callback = Callback::new_paint_callback(
        rect,
        WorkspaceRenderCallback {
            camera: app.camera.clone(),
            transform: app
                .camera
                .view_projection_matrix(rect.width() / rect.height()),

            bed_size: app.slice_config.platform_size,
            grid_size: app.config.grid_size,

            is_moving: response.dragged(),
            slice_operation: app.slice_operation.clone(),

            models: app.meshes.clone(),
            config: app.config.clone(),
            line_support_debug: app.state.line_support_debug.clone(),
        },
    );
    ui.painter().add(callback);
}
