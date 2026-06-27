use std::{cmp::Reverse, iter, sync::atomic::Ordering};

use common::{geometry::convex_hull, units::Milimeter};
use egui::{
    Button, CollapsingHeader, Color32, ComboBox, DragValue, Ui, Widget, emath::OrderedFloat, vec2,
};
use egui_plot::{Line, Plot};
use nalgebra::{Rotation3, Vector2};
use tools::auto_layout::{self, CacheEntry, Hull, LayoutCache, Objective, Placement, Rotation};

use crate::{
    app::App,
    project::model::Model,
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
    let edit = tool.running.is_none();

    grid("").show(ui, |ui| {
        ui.label("Objective");
        ui.add_enabled_ui(edit, |ui| {
            ComboBox::from_id_salt("objective")
                .selected_text(tool.config.objective.name())
                .show_ui(ui, |ui| {
                    for objective in Objective::ALL {
                        ui.selectable_value(
                            &mut tool.config.objective,
                            objective,
                            objective.name(),
                        );
                    }
                });
        });
        ui.end_row();

        ui.label("Rotation");
        ui.add_enabled_ui(edit, |ui| {
            ComboBox::from_id_salt("rotation")
                .selected_text(tool.config.rotation.name())
                .show_ui(ui, |ui| {
                    for rotation in Rotation::ALL {
                        ui.selectable_value(&mut tool.config.rotation, rotation, rotation.name());
                    }
                });
        });
        ui.end_row();

        ui.label("Padding");
        ui.add_enabled(edit, DragValue::new(&mut tool.config.padding).suffix(" mm"));
        ui.end_row();

        ui.label("Segment Steps");
        ui.horizontal(|ui| {
            ui.add_enabled(edit, DragValue::new(&mut tool.config.segment_steps));
            ui.take_available_width();
        });
        ui.end_row();
    });

    ui.add_space(8.0);
    CollapsingHeader::new("Annealing")
        .default_open(true)
        .show(ui, |ui| {
            grid("annealing").show(ui, |ui| {
                ui.label("Start Temperature");
                ui.add_enabled(edit, DragValue::new(&mut tool.config.start_temp));
                ui.end_row();

                ui.label("End Temperature");
                ui.add_enabled(edit, DragValue::new(&mut tool.config.end_temp));
                ui.end_row();

                ui.label("Iterations");
                ui.add_enabled(edit, DragValue::new(&mut tool.config.iters));
                ui.end_row();

                ui.label("Cooling");
                ui.horizontal(|ui| {
                    ui.add_enabled(edit, DragValue::new(&mut tool.config.cooling));
                    ui.take_available_width();
                });
                ui.end_row();

                ui.label("Bounds Penalty");
                ui.add_enabled(edit, DragValue::new(&mut tool.config.bounds_penalty));
                ui.end_row();
            });
        });

    ui.add_space(8.0);
    if let Some(running) = &tool.running {
        CollapsingHeader::new("Status")
            .default_open(true)
            .show(ui, |ui| {
                Plot::new("score_history")
                    .allow_drag(false)
                    .allow_zoom(false)
                    .allow_scroll(false)
                    .allow_boxed_zoom(false)
                    .show_axes([true, false])
                    .width(ui.available_width())
                    .view_aspect(2.0)
                    .show(ui, |plot| {
                        let history = running.history.lock();
                        let last = (
                            running.iteration.load(Ordering::Relaxed),
                            history.last().map(|x| x.1).unwrap_or_default(),
                        );

                        let points = (history.iter())
                            .chain(iter::once(&last))
                            .map(|(x, y)| [*x as f64, y.log2() as f64])
                            .collect::<Vec<_>>();
                        plot.add(Line::new("", points).color(Color32::WHITE));
                    });
            });

        while let Ok(result) = running.rx.try_recv() {
            (result.iter()).for_each(|x| apply_placement(&mut app.project.models, x));
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
                let (cache, models) = layout_cache(tool.config.padding, &app.project.models);

                tool.config.platform_size = platform;
                tool.models = models;
                tool.run(cache);
            }
        });
    }

    false
}

pub fn layout_cache(padding: f32, models: &[Model]) -> (LayoutCache, Vec<auto_layout::Model>) {
    let mut out = Vec::new();
    let mut cache = LayoutCache::new(padding);

    for model in models.iter().filter(|x| !x.hidden) {
        out.push(auto_layout::Model::new(
            model.id.raw(),
            model.mesh.mesh_id(),
        ));

        let entry = CacheEntry::new(model.mesh.mesh_id(), 0.0);
        cache.populate_hull(entry, || {
            let rotation = model.mesh.rotation();
            let transform = Rotation3::from_euler_angles(rotation.x, rotation.y, 0.0);
            let points = (model.mesh.vertices().iter())
                .map(|x| (transform * x).xy())
                .collect::<Vec<_>>();
            Hull::new(convex_hull(&points))
        });
    }

    out.sort_by_cached_key(|x| Reverse(OrderedFloat(area(&cache.hull(&x.entry()).hull))));
    (cache, out)
}

pub fn apply_placement(models: &mut [Model], placement: &Placement) {
    if let Some(model) = models.iter_mut().find(|x| x.id.raw() == placement.model) {
        let new_position = placement.position.xy().push(model.mesh.position().z);
        let new_rotation = model.mesh.rotation().xy().push(placement.rotation);
        model.mesh.set_position(new_position);
        model.mesh.set_rotation(new_rotation);
    }
}

fn area(polygon: &[Vector2<f32>]) -> f32 {
    let mut j = polygon.len() - 1;
    let mut area = 0.0;
    for i in 0..polygon.len() {
        area += (polygon[j].x + polygon[i].x) * (polygon[j].y - polygon[i].y);
        j = i;
    }

    area.abs() / 2.0
}
