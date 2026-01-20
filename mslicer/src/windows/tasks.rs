use egui::{Context, Grid, ProgressBar, Ui};

use crate::app::App;

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    if !app.tasks.any_with_status() {
        ui.vertical_centered(|ui| ui.label("No async tasks running."));
        return;
    }

    Grid::new("slice_config")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            for task in app.tasks.iter() {
                let Some(status) = task.status() else {
                    continue;
                };

                let res1 = ui.label(status.name);
                let res2 = ui.add(ProgressBar::new(status.progress).show_percentage());
                if let Some(details) = status.details {
                    res1.on_hover_text(&details);
                    res2.on_hover_text(details);
                }

                ui.end_row();
            }
        });
}
