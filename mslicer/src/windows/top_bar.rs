use std::collections::HashMap;
use std::f32::consts::TAU;

use const_format::concatcp;
use egui::{
    Align, Align2, Button, Context, FontId, Frame, Grid, Id, Layout, PopupAnchor, ProgressBar,
    Stroke, StrokeKind, TopBottomPanel, Ui, vec2,
};
use egui_phosphor::regular::{CARDS, FILE_TEXT, GIT_DIFF, HAMMER, HOURGLASS, STACK};

#[cfg(windows)]
use crate::system::windows::launch_install;
use crate::{
    app::App,
    project::Collection,
    task::ProjectLoad,
    ui::{components::labeled_separator, shortcuts, shortcuts::Shortcut},
    windows::{
        Tab,
        tools::{self, graphics_3d},
    },
};

pub fn ui(app: &mut App, ctx: &Context) {
    shortcuts::handle(app, ctx);

    TopBottomPanel::top("top_panel")
        .frame(Frame::side_top_panel(&ctx.style()).inner_margin(4))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.style_mut().spacing.item_spacing.x = 4.0;

                ui.add_space(4.0);
                ui.heading("mslicer");
                ui.separator();

                ui.style_mut().spacing.item_spacing.x = 6.0;
                ui.menu_button(concatcp!(FILE_TEXT, " File"), |ui| {
                    ui.set_width(150.0);

                    labeled_separator(ui, "Import");
                    menu_button(
                        (ui, app, ctx),
                        &shortcuts::IMPORT_MODEL_SHORTCUT,
                        "Load Mesh",
                    );
                    menu_button(
                        (ui, app, ctx),
                        &shortcuts::LOAD_TEAPOT_SHORTCUT,
                        "Utah Teapot",
                    );
                    menu_button(
                        (ui, app, ctx),
                        &shortcuts::IMPORT_SLICED_SHORTCUT,
                        "Load Sliced",
                    );

                    labeled_separator(ui, "Project");
                    menu_button((ui, app, ctx), &shortcuts::NEW_PROJECT_SHORTCUT, "New");
                    menu_button((ui, app, ctx), &shortcuts::LOAD_PROJECT_SHORTCUT, "Open");
                    ui.add_enabled_ui(!app.config.recent_projects.is_empty(), |ui| {
                        ui.menu_button("Recent", |ui| {
                            let mut load = None;
                            for path in app.config.recent_projects.iter() {
                                let name = path.file_name().unwrap().to_string_lossy();
                                if ui.button(name).clicked() {
                                    ui.close();
                                    load = Some(path.clone());
                                }
                            }

                            if let Some(path) = load {
                                app.tasks.add(ProjectLoad::new(path));
                            }
                        });
                    });
                    menu_button((ui, app, ctx), &shortcuts::SAVE_PROJECT_SHORTCUT, "Save");
                    ui.add_enabled_ui(app.project.path.is_some(), |ui| {
                        menu_button(
                            (ui, app, ctx),
                            &shortcuts::SAVE_AS_PROJECT_SHORTCUT,
                            "Save As",
                        )
                    });

                    labeled_separator(ui, "Misc");
                    #[cfg(windows)]
                    if app.config.portable && ui.button("Install").clicked() {
                        app.config.portable = false;
                        let _ = app.config.save(&app.config_dir);
                        launch_install().unwrap();
                    }

                    menu_button((ui, app, ctx), &shortcuts::QUIT_SHORTCUT, "Quit");
                });

                ui.menu_button(concatcp!(GIT_DIFF, " Edit"), |ui| {
                    ui.set_width(150.0);

                    labeled_separator(ui, "History");
                    ui.add_enabled_ui(app.history.can_undo(), |ui| {
                        menu_button((ui, app, ctx), &shortcuts::UNDO_SHORTCUT, "Undo")
                    });
                    ui.add_enabled_ui(app.history.can_redo(), |ui| {
                        menu_button((ui, app, ctx), &shortcuts::REDO_SHORTCUT, "Redo");
                    });

                    labeled_separator(ui, "Selections");
                    menu_button(
                        (ui, app, ctx),
                        &shortcuts::SELECT_ALL_SHORTCUT,
                        "Select Models",
                    );
                    ui.add_enabled_ui(app.state.selected.has_any(), |ui| {
                        menu_button((ui, app, ctx), &shortcuts::SELECT_NONE_SHORTCUT, "Deselect");
                    });
                });

                ui.menu_button(concatcp!(HAMMER, " Tools"), |ui| {
                    ui.set_width(150.0);
                    labeled_separator(ui, "Auto Layout");
                    menu_button((ui, app, ctx), &shortcuts::LAYOUT_SHORTCUT, "Quick Layout");
                    ui.button("Advanced Layout")
                        .clicked()
                        .then(|| tools::auto_layout::open(app));

                    labeled_separator(ui, "Generators");
                    (ui.button("Printed Circuit Board").clicked())
                        .then(|| tools::printed_circuit_board::open(app));

                    labeled_separator(ui, "Exposure");
                    (ui.button("Exposure Test").clicked()).then(|| tools::exposure_test::open(app));
                    (ui.button("Internal Exposure Test").clicked())
                        .then(|| tools::internal_exposure_test::open(app));

                    labeled_separator(ui, "Miscellaneous");
                    ui.button("Collect Instances")
                        .clicked()
                        .then(|| collect_instances(app));

                    if ui.button("3D Graphics").clicked() {
                        graphics_3d::open(app);
                    }
                });

                ui.menu_button(concatcp!(CARDS, " View"), |ui| {
                    ui.set_width(150.0);

                    labeled_separator(ui, "Actions");
                    app.config.ui.about |= ui.button("About mslicer").clicked();
                    app.state.queue_reset_ui |= ui.button("Reset Interface").clicked();

                    labeled_separator(ui, "Windows");
                    for tab in Tab::ALL {
                        app.panels.checkbox(tab, |open| {
                            ui.checkbox(open, tab.name());
                        });
                    }
                });

                ui.add_enabled_ui(!app.is_slicing(), |ui| {
                    let slice_button = ui.add(
                        Button::new(concatcp!(STACK, " Slice"))
                            .shortcut_text(ctx.format_shortcut(&shortcuts::SLICE_SHORTCUT)),
                    );
                    slice_button.clicked().then(|| app.slice());
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_space(2.0);
                    tasks_button(app, ctx, ui);
                    ui.take_available_width();
                });
            });
        });
}

fn tasks_button(app: &mut App, ctx: &Context, ui: &mut Ui) {
    let y = ui.spacing().interact_size.y;
    let (rect, mut response) = ui.allocate_exact_size(vec2(y, y), egui::Sense::click());
    response = response.on_hover_text("Monitor the progress of async background tasks.");
    app.config.ui.tasks ^= response.clicked();

    let visuals = ui
        .style()
        .interact_selectable(&response, app.config.ui.tasks);
    ui.painter().rect(
        rect,
        visuals.corner_radius,
        visuals.bg_fill,
        visuals.bg_stroke,
        StrokeKind::Outside,
    );

    let f = app.tasks.progress();
    let f_ease = ctx.animate_value_with_time(ui.id().with("progress"), f, 0.2);
    if f == 0.0 {
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            HOURGLASS,
            FontId::default(),
            visuals.text_color(),
        );
    } else {
        let stroke = Stroke::new(2.0_f32, visuals.text_color());
        let points = (0..=10).map(|i| {
            let t = i as f32 / 10.0 * TAU * f_ease;
            rect.center() + vec2(t.cos(), t.sin()) * ((y * 0.75 - stroke.width) / 2.0)
        });
        ui.painter().line(points.collect(), stroke);
    }

    let anchor = PopupAnchor::Position(response.rect.max + vec2(0.0, 4.0));
    egui::Popup::new(Id::new("tasks"), ctx.clone(), anchor, ui.layer_id())
        .open(app.config.ui.tasks && app.tasks.any_with_status())
        .show(|ui| {
            ui.set_width(300.0);
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
        });
}

fn menu_button((ui, app, ctx): (&mut Ui, &mut App, &Context), shortcut: &Shortcut, text: &str) {
    let button = Button::new(text).shortcut_text(ctx.format_shortcut(shortcut));
    if ui.add(button).clicked() {
        (shortcut.callback)(app, ctx);
        ui.close();
    }
}

fn collect_instances(app: &mut App) {
    let mut instances = HashMap::<_, Vec<_>>::new();
    for model in app.project.models.iter().filter(|x| x.collection.is_none()) {
        instances
            .entry(model.mesh.mesh_id())
            .or_default()
            .push(model.id);
    }

    for (_, v) in instances.iter().filter(|(_, v)| v.len() > 1) {
        let collection = Collection::new_unnamed();
        for model in v {
            if let Some(model) = app.project.model(*model) {
                model.collection = Some(collection.id);
            }
        }
        app.project.collections.push(collection);
    }
}
