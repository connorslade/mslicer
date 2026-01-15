use std::fs::File;

use const_format::concatcp;
use egui::{Button, Context, Key, KeyboardShortcut, Modifiers, TopBottomPanel, ViewportCommand};
use egui_phosphor::regular::STACK;

use crate::{
    app::{
        App,
        task::{FileDialog, MeshLoad},
    },
    include_asset,
};

const IMPORT_MODEL_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::I);
const LOAD_TEAPOT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::T);
const SAVE_PROJECT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::S);
const LOAD_PROJECT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::O);
const QUIT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Q);
const SLICE_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::R);

pub fn ui(app: &mut App, ctx: &Context) {
    ctx.input_mut(|x| x.consume_shortcut(&IMPORT_MODEL_SHORTCUT))
        .then(|| import_model(app));
    ctx.input_mut(|x| x.consume_shortcut(&LOAD_TEAPOT_SHORTCUT))
        .then(|| import_teapot(app));
    ctx.input_mut(|x| x.consume_shortcut(&SAVE_PROJECT_SHORTCUT))
        .then(|| save(app));
    ctx.input_mut(|x| x.consume_shortcut(&LOAD_PROJECT_SHORTCUT))
        .then(|| load(app));
    ctx.input_mut(|x| x.consume_shortcut(&QUIT_SHORTCUT))
        .then(|| quit(ctx));
    ctx.input_mut(|x| x.consume_shortcut(&SLICE_SHORTCUT))
        .then(|| app.slice());

    TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("mslicer");
            ui.separator();

            ui.menu_button("🖹 File", |ui| {
                ui.style_mut().visuals.button_frame = false;
                ui.set_width(170.0);

                let import_model_button = ui.add(
                    Button::new("Import Model")
                        .shortcut_text(ctx.format_shortcut(&IMPORT_MODEL_SHORTCUT)),
                );
                import_model_button.clicked().then(|| import_model(app));

                let import_teapot_button = ui.add(
                    Button::new("Load Utah Teapot")
                        .shortcut_text(ctx.format_shortcut(&LOAD_TEAPOT_SHORTCUT)),
                );
                import_teapot_button.clicked().then(|| import_teapot(app));

                ui.separator();

                let save_project_button = ui.add(
                    Button::new("Save Project")
                        .shortcut_text(ctx.format_shortcut(&SAVE_PROJECT_SHORTCUT)),
                );
                save_project_button.clicked().then(|| save(app));

                let load_project_button = ui.add(
                    Button::new("Load Project")
                        .shortcut_text(ctx.format_shortcut(&LOAD_PROJECT_SHORTCUT)),
                );
                load_project_button.clicked().then(|| load(app));

                ui.add_enabled_ui(!app.config.recent_projects.is_empty(), |ui| {
                    ui.menu_button("Recent Projects", |ui| {
                        let mut load = None;
                        for path in app.config.recent_projects.iter() {
                            let name = path.file_name().unwrap().to_string_lossy();
                            if ui.button(name).clicked() {
                                ui.close();
                                load = Some(path.clone());
                            }
                        }

                        if let Some(load) = load {
                            app.load_project(&load);
                        }
                    });
                });

                ui.separator();

                let quit_button =
                    ui.add(Button::new("Quit").shortcut_text(ctx.format_shortcut(&QUIT_SHORTCUT)));
                quit_button.clicked().then(|| quit(ctx));

                // Close the menu if a button is clicked
                for button in [
                    import_model_button,
                    import_teapot_button,
                    save_project_button,
                    load_project_button,
                    quit_button,
                ] {
                    button.clicked().then(|| ui.close());
                }
            });

            let slicing = match &app.slice_operation {
                Some(operation) => operation.progress.completed() < operation.progress.total(),
                None => false,
            };
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

fn import_model(app: &mut App) {
    app.tasks.add(FileDialog::pick_file(
        ("Mesh", &["stl", "obj"]),
        |app, path| {
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            let ext = path.extension();
            let format = ext.unwrap_or_default().to_string_lossy();

            let file = File::open(&path).unwrap();
            app.tasks.add(MeshLoad::file(file, name, &format));
        },
    ));
}

fn import_teapot(app: &mut App) {
    app.tasks.add(MeshLoad::buffer(
        include_asset!("teapot.stl"),
        "Utah Teapot".into(),
        "stl",
    ));
}

fn save(app: &mut App) {
    app.tasks.add(FileDialog::save_file(
        ("mslicer project", &["mslicer"]),
        |app, path| app.save_project(&path.with_extension("mslicer")),
    ));
}

fn load(app: &mut App) {
    app.tasks.add(FileDialog::save_file(
        ("mslicer project", &["mslicer"]),
        |app, path| app.load_project(&path),
    ));
}

fn quit(ctx: &Context) {
    ctx.send_viewport_cmd(ViewportCommand::Close)
}
