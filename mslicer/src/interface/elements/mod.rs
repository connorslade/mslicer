use egui::Context;

use crate::core::App;

pub mod drag_and_drop;
pub mod top_bar;
pub mod welcome;

pub fn ui(app: &mut App, ctx: &Context) {
    drag_and_drop::update(app, ctx);
    welcome::ui(app, ctx);
    top_bar::ui(app, ctx);
}
