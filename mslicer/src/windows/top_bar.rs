use std::{f32::consts::TAU, fs::File};

use const_format::concatcp;
use egui::{
    Align, Align2, Button, Context, FontId, Frame, Grid, Id, Key, KeyboardShortcut, Layout,
    Modifiers, PopupAnchor, ProgressBar, Stroke, StrokeKind, TopBottomPanel, Ui, ViewportCommand,
    vec2,
};
use egui_phosphor::regular::{CARDS, FILE_TEXT, GIT_DIFF, HAMMER, HOURGLASS, STACK};

use crate::{
    app::{
        App,
        project::Project,
        task::{FileDialog, MeshLoad, ProjectLoad},
    },
    include_asset,
    ui::{components::labeled_separator, popup::Popup},
    windows::{Tab, tools},
};

const COMMAND_SHIFT: Modifiers = Modifiers::COMMAND.plus(Modifiers::SHIFT);

const IMPORT_MODEL_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::I);
const LOAD_TEAPOT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::T);
const SAVE_PROJECT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::S);
const SAVE_AS_PROJECT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(COMMAND_SHIFT, Key::S);
const NEW_PROJECT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::N);
const LOAD_PROJECT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
const QUIT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Q);
const SLICE_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::R);
const UNDO_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Z);
const REDO_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Y);

type ShortcutCallback = fn(&mut App, &Context);
const SHORTCUTS: &[(KeyboardShortcut, ShortcutCallback)] = &[
    (IMPORT_MODEL_SHORTCUT, |app, _ctx| import_model(app)),
    (LOAD_TEAPOT_SHORTCUT, |app, _ctx| import_teapot(app)),
    (NEW_PROJECT_SHORTCUT, |app, _ctx| new(app)),
    (LOAD_PROJECT_SHORTCUT, |app, _ctx| load(app)),
    (SAVE_PROJECT_SHORTCUT, |app, _ctx| save(app)),
    (SAVE_AS_PROJECT_SHORTCUT, |app, _ctx| save_as(app)),
    (QUIT_SHORTCUT, |_app, ctx| quit(ctx)),
    (UNDO_SHORTCUT, |app, _| app.history().undo()),
    (REDO_SHORTCUT, |app, _| app.history().redo()),
    (SLICE_SHORTCUT, |app, _ctx| app.slice()),
];

pub fn ui(app: &mut App, ctx: &Context) {
    for (shortcut, callback) in SHORTCUTS {
        ctx.input_mut(|x| x.consume_shortcut(shortcut))
            .then(|| callback(app, ctx));
    }

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

                    labeled_separator(ui, "Model");
                    menu_button((ui, app, ctx), SHORTCUTS[0], "Import Mesh");
                    menu_button((ui, app, ctx), SHORTCUTS[1], "Utah Teapot");

                    labeled_separator(ui, "Project");
                    menu_button((ui, app, ctx), SHORTCUTS[2], "New");
                    menu_button((ui, app, ctx), SHORTCUTS[3], "Open");
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
                    menu_button((ui, app, ctx), SHORTCUTS[4], "Save");
                    ui.add_enabled_ui(app.project.path.is_some(), |ui| {
                        menu_button((ui, app, ctx), SHORTCUTS[5], "Save As")
                    });

                    labeled_separator(ui, "Misc");
                    menu_button((ui, app, ctx), SHORTCUTS[6], "Quit");
                });

                ui.menu_button(concatcp!(GIT_DIFF, " Edit"), |ui| {
                    ui.set_width(150.0);

                    labeled_separator(ui, "History");
                    ui.add_enabled_ui(app.history.can_undo(), |ui| {
                        menu_button((ui, app, ctx), SHORTCUTS[7], "Undo")
                    });
                    ui.add_enabled_ui(app.history.can_redo(), |ui| {
                        menu_button((ui, app, ctx), SHORTCUTS[8], "Redo");
                    });
                });

                ui.menu_button(concatcp!(HAMMER, " Tools"), |ui| {
                    ui.set_width(150.0);
                    labeled_separator(ui, "Generators");
                    (ui.button("Exposure Test").clicked()).then(|| tools::exposure_test::open(app));
                });

                ui.menu_button(concatcp!(CARDS, " View"), |ui| {
                    ui.set_width(150.0);

                    labeled_separator(ui, "Actions");
                    app.config.about |= ui.button("About mslicer").clicked();
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
                            .shortcut_text(ctx.format_shortcut(&SLICE_SHORTCUT)),
                    );
                    slice_button.clicked().then(|| app.slice());
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_space(2.0);
                    tasks_button(app, ctx, ui);
                    ui.take_available_width();
                })
            });
        });
}

fn tasks_button(app: &mut App, ctx: &Context, ui: &mut Ui) {
    let y = ui.spacing().interact_size.y;
    let (rect, mut response) = ui.allocate_exact_size(vec2(y, y), egui::Sense::click());
    response = response.on_hover_text("Monitor the progress of async background tasks.");
    app.config.tasks ^= response.clicked();

    let visuals = ui.style().interact_selectable(&response, app.config.tasks);
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
        let stroke = Stroke::new(2.0, visuals.text_color());
        let points = (0..=10).map(|i| {
            let t = i as f32 / 10.0 * TAU * f_ease;
            rect.center() + vec2(t.cos(), t.sin()) * ((y * 0.75 - stroke.width) / 2.0)
        });
        ui.painter().line(points.collect(), stroke);
    }

    let anchor = PopupAnchor::Position(response.rect.max + vec2(0.0, 4.0));
    egui::Popup::new(Id::new("tasks"), ctx.clone(), anchor, ui.layer_id())
        .open(app.config.tasks && app.tasks.any_with_status())
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

fn menu_button(
    (ui, app, ctx): (&mut Ui, &mut App, &Context),
    (shortcut, callback): (KeyboardShortcut, ShortcutCallback),
    text: &str,
) {
    let button = Button::new(text).shortcut_text(ctx.format_shortcut(&shortcut));
    if ui.add(button).clicked() {
        callback(app, ctx);
        ui.close();
    }
}

fn import_model(app: &mut App) {
    app.tasks.add(FileDialog::pick_file(
        ("Mesh", &["stl", "obj"]),
        |_app, path, tasks| {
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            let ext = path.extension();
            let format = ext.unwrap_or_default().to_string_lossy();

            let file = File::open(path).unwrap();
            tasks.push(Box::new(MeshLoad::file(file, name, format.into())));
        },
    ));
}

fn import_teapot(app: &mut App) {
    app.tasks.add(MeshLoad::buffer(
        include_asset!("teapot.stl"),
        "Utah Teapot".into(),
        "stl".into(),
    ));
}

fn new(app: &mut App) {
    if !app.project.models.is_empty() {
        app.popup.open(Popup::new("Modified File", |app, ui| {
            ui.add_space(8.0);
            if app.project.path.is_some() {
                ui.label("Do you want to save the changes made to this project?");
            } else {
                ui.label("Do you want to save this project?");
            }
            ui.add_space(8.0);

            let mut close = false;
            ui.columns(2, |ui| {
                ui[0].centered_and_justified(|ui| {
                    if ui.button("Don't Save").clicked() {
                        app.project.reset(&app.config.default_slice_config);
                        close = true;
                    }
                });
                ui[1].centered_and_justified(|ui| {
                    if ui.button("Save").clicked() {
                        app.tasks.add_boxed(app.project.save());
                    }
                });
            });

            close
        }));
    }
}

fn save(app: &mut App) {
    let task = app.project.save();
    app.tasks().add_boxed(task);
}

fn save_as(app: &mut App) {
    let task = app.project.save_as();
    app.tasks().add(task);
}

fn load(app: &mut App) {
    app.tasks.add(Project::load());
}

fn quit(ctx: &Context) {
    ctx.send_viewport_cmd(ViewportCommand::Close)
}
