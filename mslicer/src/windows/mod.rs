use std::mem;

use egui::{CentralPanel, Color32, Context, Frame, Id, Key, Sense, Theme, Ui, WidgetText};
use egui_dock::{DockArea, NodeIndex, SurfaceIndex, TabViewer};
use egui_wgpu::Callback;
use nalgebra::Matrix4;
use parking_lot::MappedRwLockWriteGuard;
use serde::{Deserialize, Serialize};
use slicer::supports::route_support;

use crate::{app::App, render::workspace::WorkspaceRenderCallback};

mod about;
mod logs;
mod models;
mod remote_print;
mod slice_config;
mod slice_operation;
mod supports;
mod top_bar;
mod workspace;

struct Tabs<'a> {
    app: &'a mut App,
    ctx: &'a Context,
}

#[derive(Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub enum Tab {
    About,
    Logs,
    Models,
    RemotePrint,
    SliceConfig,
    SliceOperation,
    Supports,
    Viewport,
    Workspace,
}

impl Tab {
    const ALL: [Tab; 8] = [
        Tab::About,
        Tab::Logs,
        Tab::Models,
        Tab::RemotePrint,
        Tab::SliceConfig,
        Tab::SliceOperation,
        Tab::Supports,
        Tab::Workspace,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Tab::About => "About",
            Tab::Logs => "Logs",
            Tab::Models => "Models",
            Tab::RemotePrint => "Remote Print",
            Tab::SliceConfig => "Slice Config",
            Tab::SliceOperation => "Slice Operation",
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
            Tab::About => about::ui(self.app, ui, self.ctx),
            Tab::Logs => logs::ui(self.app, ui, self.ctx),
            Tab::Models => models::ui(self.app, ui, self.ctx),
            Tab::RemotePrint => remote_print::ui(self.app, ui, self.ctx),
            Tab::SliceConfig => slice_config::ui(self.app, ui, self.ctx),
            Tab::SliceOperation => slice_operation::ui(self.app, ui, self.ctx),
            Tab::Supports => supports::ui(self.app, ui, self.ctx),
            Tab::Viewport => viewport(self.app, ui, self.ctx),
            Tab::Workspace => workspace::ui(self.app, ui, self.ctx),
        }
    }

    fn add_popup(&mut self, ui: &mut Ui, surface: SurfaceIndex, node: NodeIndex) {
        ui.set_min_width(120.0);
        ui.style_mut().visuals.button_frame = false;

        let dock_state = &mut self.app.dock_state;

        for tab in Tab::ALL {
            let already_open = dock_state.find_tab(&tab).is_some();
            if !already_open && ui.button(tab.name()).clicked() {
                if let Some(surface) = dock_state.get_surface_mut(surface) {
                    let tree = surface.node_tree_mut().unwrap();
                    tree.set_focused_node(node);
                    tree.push_to_focused_leaf(tab);
                } else {
                    dock_state.add_window(vec![tab]);
                }
            }
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
    top_bar::ui(app, ctx);

    mem::take(&mut app.state.queue_reset_ui).then(|| app.reset_ui());
    CentralPanel::default().frame(Frame::NONE).show(ctx, |ui| {
        // i am once again too tired to deal with this
        let dock_state = unsafe { &mut *(&mut app.dock_state as *mut _) };
        DockArea::new(dock_state)
            .show_add_buttons(true)
            .show_add_popup(true)
            .show_leaf_close_all_buttons(false)
            .show_leaf_collapse_buttons(false)
            .tab_context_menus(false)
            .show_inside(ui, &mut Tabs { app, ctx });
    });
}

fn viewport(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::drag());
    let hover_pos = response.hover_pos().unwrap_or_default();
    let aspect = rect.width() / rect.height();
    let uv = (hover_pos - rect.min) / rect.size();

    let (pos, dir) = app.camera.hovered_ray(aspect, (uv.x, uv.y));

    if ui.input(|i| i.key_pressed(Key::S)) {
        let lines = &mut app.state.line_support_debug;
        let mut line = |a, b| lines.push([a, b - a]);

        for model in app.models.read().iter() {
            let Some(intersection) = model.bvh.intersect_ray(
                &model.mesh,
                model.mesh.inv_transform(&pos),
                model.mesh.inv_transform_normal(&dir),
            ) else {
                continue;
            };

            let normal = (model.mesh).transform_normal(&model.mesh.normal(intersection.face));
            let start = intersection.position + normal * 0.1;
            if let Some(lines) = route_support(&model.mesh, &model.bvh, start) {
                line(intersection.position, start);
                line(lines[0], lines[1]);
                line(lines[1], lines[2]);
            }
        }
    }

    app.camera.handle_movement(&response, ui);

    let color = match app.config.theme {
        Theme::Dark => Color32::from_rgb(9, 9, 9),
        Theme::Light => Color32::from_rgb(255, 255, 255),
    };
    ui.painter().rect_filled(rect, 0.0, color);

    let view_projection = app.camera.view_projection_matrix(aspect);

    let callback = Callback::new_paint_callback(
        rect,
        app.get_workspace_render_callback(view_projection, response.dragged()),
    );

    ui.painter().add(callback);
}

impl App {
    pub fn get_workspace_render_callback(
        &self,
        view_projection: Matrix4<f32>,
        is_moving: bool,
    ) -> WorkspaceRenderCallback {
        let (show_overhang, overhang_angle) = self.config.overhang_visualization;
        WorkspaceRenderCallback {
            camera: self.camera.clone(),
            transform: view_projection,

            bed_size: self.slice_config.platform_size,
            grid_size: self.config.grid_size,

            is_moving,
            models: self.models.clone(),
            config: self.config.clone(),

            line_support_debug: self.state.line_support_debug.clone(),
            overhang_angle: show_overhang.then_some(overhang_angle),
        }
    }

    pub fn get_callback_resource_mut<T: 'static>(&self) -> MappedRwLockWriteGuard<'_, T> {
        MappedRwLockWriteGuard::map(self.render_state.renderer.write(), |x| {
            x.callback_resources.get_mut::<T>().unwrap()
        })
    }
}
