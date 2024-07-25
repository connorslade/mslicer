use egui::{Context, Ui};

use crate::app::App;

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    ui.add(egui_tracing::Logs::new(app.state.event_collector.clone()));
}
