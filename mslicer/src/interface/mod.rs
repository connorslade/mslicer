use std::mem;

use egui::{CentralPanel, Context, Frame};
use egui_dock::DockArea;

use crate::{core::App, interface::panels::Tabs};

pub mod components;
pub mod drag_and_drop;
pub mod panels;
pub mod popup;
pub mod shortcuts;
pub mod tools;
pub mod top_bar;
pub mod welcome;

pub fn ui(app: &mut App, ctx: &Context) {
    drag_and_drop::update(app, ctx);
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
