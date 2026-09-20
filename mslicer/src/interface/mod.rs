use std::mem;

use egui::{CentralPanel, Context, Frame};
use egui_dock::DockArea;

use crate::{core::App, interface::panels::Tabs};

pub mod components;
pub mod elements;
pub mod panels;
pub mod popup;
pub mod shortcuts;
pub mod tools;

pub fn ui(app: &mut App, ctx: &Context) {
    elements::ui(app, ctx);

    mem::take(&mut app.state.queue_reset_ui).then(|| app.panels.reset_ui());
    CentralPanel::default().frame(Frame::NONE).show(ctx, |ui| {
        let mut dock_state = app.panels.take_inner();
        DockArea::new(&mut dock_state)
            .show_leaf_close_all_buttons(false)
            .show_leaf_collapse_buttons(false)
            .tab_context_menus(false)
            .show_inside(ui, &mut Tabs { app, ctx });
        app.panels.replace_inner(dock_state);
    });
}
