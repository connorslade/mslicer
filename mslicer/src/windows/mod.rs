use std::mem;

use egui::{
    CentralPanel, Color32, Context, Frame, Id, Painter, Rect, Sense, Stroke, StrokeKind, Theme, Ui,
    WidgetText, pos2, vec2,
};
use egui_dock::{DockArea, TabViewer};
use egui_wgpu::Callback;
use nalgebra::Matrix4;
use serde::{Deserialize, Serialize};

use crate::{
    app::App,
    render::{interface::basis::BasisRenderCallback, workspace::WorkspaceRenderCallback},
    ui::state::WorkspaceHover,
    windows::supports::manual_support_placement,
};

mod logs;
mod models;
mod remote_print;
mod slice_config;
mod sliced;
mod supports;
pub mod tools;
mod top_bar;
mod welcome;
mod workspace;

struct Tabs<'a> {
    app: &'a mut App,
    ctx: &'a Context,
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tab {
    Logs,
    Models,
    RemotePrint,
    SliceConfig,
    Sliced,
    Supports,
    Viewport,
    Workspace,
}

impl Tab {
    const ALL: [Tab; 7] = [
        Tab::Logs,
        Tab::Models,
        Tab::RemotePrint,
        Tab::SliceConfig,
        Tab::Sliced,
        Tab::Supports,
        Tab::Workspace,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Tab::Logs => "Logs",
            Tab::Models => "Models",
            Tab::RemotePrint => "Remote Print",
            Tab::SliceConfig => "Slice Config",
            Tab::Sliced => "Sliced",
            Tab::Supports => "Supports",
            Tab::Viewport => "Viewport",
            Tab::Workspace => "Workspace",
        }
    }
}

impl TabViewer for Tabs<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.name().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::Logs => logs::ui(self.app, ui, self.ctx),
            Tab::Models => models::ui(self.app, ui, self.ctx),
            Tab::RemotePrint => remote_print::ui(self.app, ui, self.ctx),
            Tab::SliceConfig => slice_config::ui(self.app, ui, self.ctx),
            Tab::Sliced => sliced::ui(self.app, ui, self.ctx),
            Tab::Supports => supports::ui(self.app, ui, self.ctx),
            Tab::Viewport => viewport(self.app, ui, self.ctx),
            Tab::Workspace => workspace::ui(self.app, ui, self.ctx),
        }
    }

    fn is_closeable(&self, tab: &Self::Tab) -> bool {
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
    welcome::ui(app, ctx);
    top_bar::ui(app, ctx);

    mem::take(&mut app.state.queue_reset_ui).then(|| app.panels.reset_ui());
    CentralPanel::default().frame(Frame::NONE).show(ctx, |ui| {
        // i am once again too tired to deal with this (todo!)
        let dock_state = unsafe { &mut *(&mut app.panels.dock_state as *mut _) };
        DockArea::new(dock_state)
            .show_leaf_close_all_buttons(false)
            .show_leaf_collapse_buttons(false)
            .tab_context_menus(false)
            .show_inside(ui, &mut Tabs { app, ctx });
    });
}

fn viewport(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
    app.camera.handle_movement(&response, ui);

    let focused = ui.input(|i| i.focused);
    let mut is_moving = app.spacenav().handle_movement(focused);
    if is_moving {
        app.state.move_timeout = ((0.025 / app.fps.frame_time()).round() as u32).min(60);
    } else if app.state.move_timeout > 0 {
        app.state.move_timeout -= 1;
        is_moving = true;
    }
    is_moving |= response.dragged();

    let aspect = rect.width() / rect.height();
    let uv = (response.hover_pos().unwrap_or_default() - rect.min) / rect.size();
    app.state.workspace = WorkspaceHover::new(is_moving, aspect, uv);

    if response.clicked() && !is_moving {
        if app.state.support_placement {
            manual_support_placement(app, true);
        } else if let Some(id) = app.hovered_model() {
            app.state
                .selected
                .model_clicked(id, ui.input(|x| x.modifiers.shift));
        }
    }

    let painter = ui.painter();
    let color = match app.config.ui.theme {
        Theme::Dark => Color32::from_rgb(9, 9, 9),
        Theme::Light => Color32::from_rgb(255, 255, 255),
    };
    painter.rect_filled(rect, 0.0, color);

    painter.add(Callback::new_paint_callback(
        rect,
        app.get_workspace_render_callback(),
    ));

    paint_basis_vectors(painter, app, &rect);
}

fn paint_basis_vectors(painter: &Painter, app: &mut App, rect: &Rect) {
    let size = app.config.render.basis_size;
    if size == 0.0 {
        return;
    }

    let pad = 4.0;

    let color_bg = Color32::from_rgba_unmultiplied(0, 0, 0, 90);
    let color_edge = Color32::from_rgba_unmultiplied(0, 0, 0, 180);
    let stroke = Stroke::new(2.0, color_edge);

    let rect = Rect::from_min_size(
        pos2(rect.max.x - size - pad, rect.min.y + pad),
        vec2(size, size),
    );
    painter.rect_filled(rect, size / 2.0, color_bg);
    painter.rect_stroke(rect, size / 2.0, stroke, StrokeKind::Outside);

    painter.add(Callback::new_paint_callback(
        rect.expand(-pad),
        BasisRenderCallback {
            camera: app.camera.clone(),
        },
    ));
}

impl App {
    pub fn get_workspace_render_callback(&mut self) -> WorkspaceRenderCallback {
        WorkspaceRenderCallback {
            app: self as *mut _,
        }
    }

    pub fn view_projection(&self) -> Matrix4<f32> {
        let aspect = self.state.workspace.aspect;
        self.camera
            .view_projection_matrix(self.config.render.projection, aspect)
    }
}
