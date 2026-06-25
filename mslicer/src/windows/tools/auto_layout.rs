use common::{geometry::convex_hull, units::Milimeter};
use egui::{Button, CollapsingHeader, Color32, DragValue, Ui, Widget, vec2};
use egui_plot::{Line, Plot};
use nalgebra::Vector2;
use slicer::mesh::Mesh;
use tools::auto_layout;

use crate::{
    app::App,
    ui::{
        components::grid,
        popup::{Popup, PopupApp},
    },
};

pub const DESCRIPTION: &str = "Automatically lays out models on the print bed. Slower than the Quick Layout, but produces better results.";

pub fn open(app: &mut App) {
    app.popup
        .open(Popup::new("Auto Layout", interface).close_button(true));
}

fn interface(app: &mut PopupApp, ui: &mut Ui) -> bool {
    ui.label(DESCRIPTION);
    ui.add_space(8.0);

    let tool = &mut app.state.tools.advanced_layout;

    ui.add_enabled_ui(tool.running.is_none(), |ui| {
        grid("").show(ui, |ui| {
            ui.label("Padding");
            DragValue::new(&mut tool.config.padding)
                .suffix(" mm")
                .ui(ui);
            ui.end_row();

            ui.label("Segment Steps");
            DragValue::new(&mut tool.config.segment_steps).ui(ui);
            ui.end_row();

            ui.label("Rotation Step");
            ui.horizontal(|ui| {
                DragValue::new(&mut 0.0)
                    .suffix("°")
                    .speed(5.0)
                    .range(5.0..=180.0)
                    .ui(ui);
                ui.take_available_width();
            });
            ui.end_row();
        });
    });

    ui.add_space(8.0);
    CollapsingHeader::new("Annealing")
        .default_open(true)
        .show(ui, |ui| {
            ui.add_enabled_ui(tool.running.is_none(), |ui| {
                grid("annealing").show(ui, |ui| {
                    ui.label("Start Temperature");
                    DragValue::new(&mut tool.config.start_temp).ui(ui);
                    ui.end_row();

                    ui.label("End Temperature");
                    DragValue::new(&mut tool.config.end_temp).ui(ui);
                    ui.end_row();

                    ui.label("Iterations");
                    DragValue::new(&mut tool.config.iters).ui(ui);
                    ui.end_row();

                    ui.label("Cooling");
                    ui.horizontal(|ui| {
                        DragValue::new(&mut tool.config.cooling).ui(ui);
                        ui.take_available_width();
                    });
                    ui.end_row();
                });
            });
        });

    ui.add_space(8.0);
    if let Some(running) = &tool.running {
        CollapsingHeader::new("Status")
            .default_open(true)
            .show(ui, |ui| {
                Plot::new("score_history")
                    .width(ui.available_width())
                    .view_aspect(2.0)
                    .show(ui, |plot| {
                        let history = running.history.lock();
                        let points = (history.iter().enumerate())
                            .map(|(i, x)| [i as f64, *x as f64])
                            .collect::<Vec<_>>();

                        let mut best = Vec::new();
                        let mut best_val = f32::MAX;
                        for (i, &point) in history.iter().enumerate() {
                            if point < best_val {
                                best_val = point;
                                best.push([i as f64, point as f64]);
                            }
                        }
                        best.push([history.len().saturating_sub(1) as f64, best_val as f64]);

                        plot.add(Line::new("", points).color(Color32::WHITE));
                        plot.add(Line::new("", best).color(Color32::RED));
                    });
            });

        while let Ok(result) = running.rx.try_recv() {
            for (model, new_pos) in result.iter() {
                if let Some(model) = app.project.models.iter_mut().find(|x| x.id == *model) {
                    model.mesh.set_position(*new_pos);
                }
            }
        }

        ui.vertical_centered(|ui| {
            let button = Button::new("Stop")
                .min_size(vec2(ui.available_width(), 0.0))
                .ui(ui);
            button.clicked().then(|| tool.stop());
        });
    } else {
        ui.vertical_centered(|ui| {
            let button = Button::new("Start")
                .min_size(vec2(ui.available_width(), 0.0))
                .ui(ui);

            if button.clicked() {
                let platform =
                    (app.project.slice_config.platform_size.xy()).map(|x| x.get::<Milimeter>());
                let models = (app.project.models.iter().filter(|x| !x.hidden))
                    .map(|x| {
                        let points = project_down(&x.mesh);
                        auto_layout::Model::new(x.id, x.mesh.position(), convex_hull(&points))
                    })
                    .collect::<Vec<_>>();

                tool.config.platform_size = platform;
                tool.models = models;
                tool.run();
            }
        });
    }

    false
}

fn project_down(mesh: &Mesh) -> Vec<Vector2<f32>> {
    mesh.vertices()
        .iter()
        .map(|x| mesh.transform(&x).xy())
        .collect::<Vec<_>>()
}
