use eframe::Theme;
use egui::{CentralPanel, Color32, Context, Frame, Sense, Ui, WidgetText};
use egui_dock::{DockArea, TabViewer};
use egui_wgpu::Callback;

use crate::{app::App, render::workspace::WorkspaceRenderCallback};

mod about;
mod models;
mod slice_config;
mod slice_operation;
mod stats;
mod top_bar;
mod workspace;

pub fn ui(app: &mut App, ctx: &Context) {
    top_bar::ui(app, ctx);

    CentralPanel::default()
        .frame(Frame::none())
        .show(ctx, |ui| {
            // i am once again too tired to deal with this
            let dock_state = unsafe { &mut *(&mut app.dock_state as *mut _) };
            DockArea::new(dock_state).show_inside(ui, &mut Tabs { app, ctx });
        });
}

struct Tabs<'a> {
    app: &'a mut App,
    ctx: &'a Context,
}

pub enum Tab {
    About,
    Models,
    SliceConfig,
    Stats,
    Workspace,
    SliceOperation,
    Viewport,
}

impl Tab {
    pub fn name(&self) -> &'static str {
        match self {
            Tab::About => "About",
            Tab::Models => "Models",
            Tab::SliceConfig => "Slice Config",
            Tab::Stats => "Stats",
            Tab::Workspace => "Workspace",
            Tab::SliceOperation => "Slice Operation",
            Tab::Viewport => "Viewport",
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
            Tab::Models => models::ui(self.app, ui, self.ctx),
            Tab::SliceConfig => slice_config::ui(self.app, ui, self.ctx),
            Tab::SliceOperation => slice_operation::ui(self.app, ui, self.ctx),
            Tab::Stats => stats::ui(self.app, ui, self.ctx),
            Tab::Viewport => viewport(self.app, ui, self.ctx),
            Tab::Workspace => workspace::ui(self.app, ui, self.ctx),
        }
    }
}

fn viewport(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::drag());
    app.camera.handle_movement(&response, ui);

    let color = match app.theme {
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
            grid_size: app.grid_size,

            is_moving: response.dragged(),
            slice_operation: app.slice_operation.clone(),

            models: app.meshes.clone(),
            render_style: app.render_style,
            theme: app.theme,
        },
    );
    ui.painter().add(callback);
}
