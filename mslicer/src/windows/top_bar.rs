use std::fs::File;

use const_format::concatcp;
use egui::{
    Button, Context, Key, KeyboardShortcut, Modifiers, TopBottomPanel, Ui, ViewportCommand,
};
use egui_phosphor::regular::{FILE_TEXT, GIT_DIFF, STACK};

use crate::{
    app::{
        App,
        task::{FileDialog, MeshLoad, ProjectLoad, ProjectSave},
    },
    include_asset,
    ui::components::labeled_separator,
};

const CTRL_SHIFT: Modifiers = Modifiers::CTRL.plus(Modifiers::SHIFT);

const IMPORT_MODEL_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::I);
const LOAD_TEAPOT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::T);
const SAVE_PROJECT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::S);
const SAVE_AS_PROJECT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(CTRL_SHIFT, Key::S);
const LOAD_PROJECT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::O);
const QUIT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Q);
const SLICE_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::R);
const UNDO_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Z);
const REDO_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Y);

type ShortcutCallback = fn(&mut App, &Context);
const SHORTCUTS: &[(KeyboardShortcut, ShortcutCallback)] = &[
    (IMPORT_MODEL_SHORTCUT, |app, _ctx| import_model(app)),
    (LOAD_TEAPOT_SHORTCUT, |app, _ctx| import_teapot(app)),
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

    TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("mslicer");
            ui.separator();

            ui.menu_button(concatcp!(FILE_TEXT, " File"), |ui| {
                ui.set_width(150.0);

                labeled_separator(ui, "Model");
                menu_button((ui, app, ctx), SHORTCUTS[0], "Import Mesh");
                menu_button((ui, app, ctx), SHORTCUTS[1], "Utah Teapot");

                labeled_separator(ui, "Project");
                menu_button((ui, app, ctx), SHORTCUTS[2], "Open");
                menu_button((ui, app, ctx), SHORTCUTS[3], "Save");
                ui.add_enabled_ui(app.project.path.is_some(), |ui| {
                    menu_button((ui, app, ctx), SHORTCUTS[4], "Save As")
                });

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

                labeled_separator(ui, "Misc");
                menu_button((ui, app, ctx), SHORTCUTS[5], "Quit");
            });

            ui.menu_button(concatcp!(GIT_DIFF, " Edit"), |ui| {
                ui.set_width(150.0);

                labeled_separator(ui, "History");
                ui.add_enabled_ui(app.history.can_undo(), |ui| {
                    menu_button((ui, app, ctx), SHORTCUTS[6], "Undo")
                });
                ui.add_enabled_ui(app.history.can_redo(), |ui| {
                    menu_button((ui, app, ctx), SHORTCUTS[7], "Redo");
                });
            });

            let slicing = (app.slice_operation.as_ref())
                .map(|x| !x.progress.complete())
                .unwrap_or_default();
            ui.add_enabled_ui(!slicing, |ui| {
                let slice_button = ui.add(
                    Button::new(concatcp!(STACK, " Slice"))
                        .shortcut_text(ctx.format_shortcut(&SLICE_SHORTCUT)),
                );
                slice_button.clicked().then(|| app.slice());
            });
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

fn save(app: &mut App) {
    if let Some(path) = app.project.path.clone() {
        let project = app.project.clone();
        app.tasks()
            .add(ProjectSave::new(project, path.to_path_buf()));
    } else {
        save_as(app);
    }
}

fn save_as(app: &mut App) {
    app.tasks.add(FileDialog::save_file(
        ("mslicer project", &["mslicer"]),
        |app, path, tasks| {
            let path = path.with_extension("mslicer");
            tasks.push(Box::new(ProjectSave::new(
                app.project.clone(),
                path.to_path_buf(),
            )));
            app.project.path = Some(path);
        },
    ));
}

fn load(app: &mut App) {
    app.tasks.add(FileDialog::pick_file(
        ("mslicer project", &["mslicer"]),
        |_app, path, tasks| tasks.push(Box::new(ProjectLoad::new(path.to_path_buf()))),
    ));
}

fn quit(ctx: &Context) {
    ctx.send_viewport_cmd(ViewportCommand::Close)
}
