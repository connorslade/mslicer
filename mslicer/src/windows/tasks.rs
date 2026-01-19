use egui::{Context, Grid, Layout, ProgressBar, Ui};

use crate::app::App;

pub fn ui(app: &mut App, ui: &mut Ui, _ctx: &Context) {
    ui.with_layout(Layout::bottom_up(egui::Align::Min), |ui| {
        let mut is_empty = true;
        Grid::new("slice_config")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                for task in app.tasks.iter() {
                    let Some(status) = task.status() else {
                        continue;
                    };

                    is_empty = false;
                    ui.label(status.name);
                    ui.add(ProgressBar::new(status.progress).show_percentage());
                    ui.end_row();
                }
            });

        if is_empty {
            ui.vertical_centered(|ui| ui.label("No async tasks running."));
        }
    });
}
