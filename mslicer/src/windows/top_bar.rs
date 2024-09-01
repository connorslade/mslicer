use std::{
    fs::File,
    io::{BufReader, Cursor},
    process,
};

use const_format::concatcp;
use egui::{Button, Context, Key, KeyboardShortcut, Modifiers, TopBottomPanel};
use egui_phosphor::regular::STACK;
use rfd::FileDialog;
use tracing::error;

use crate::{
    app::App,
    ui::popup::{Popup, PopupIcon},
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
        .then(quit);
    ctx.input_mut(|x| x.consume_shortcut(&SLICE_SHORTCUT))
        .then(|| app.slice());

    TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("mslicer");
            ui.separator();

            ui.menu_button("🖹 File", |ui| {
                ui.style_mut().visuals.button_frame = false;
                ui.set_width(160.0);

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

                ui.separator();

                let quit_button =
                    ui.add(Button::new("Quit").shortcut_text(ctx.format_shortcut(&QUIT_SHORTCUT)));
                quit_button.clicked().then(quit);

                // Close the menu if a button is clicked
                for button in [
                    import_model_button,
                    import_teapot_button,
                    save_project_button,
                    load_project_button,
                    quit_button,
                ] {
                    button.clicked().then(|| ui.close_menu());
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
    // TODO: async
    if let Some(path) = FileDialog::new()
        .add_filter("Mesh", &["stl", "obj"])
        .pick_file()
    {
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let ext = path.extension();
        let format = ext.unwrap_or_default().to_string_lossy();

        let file = File::open(&path).unwrap();
        let mut buf = BufReader::new(file);
        app.load_mesh(&mut buf, &format, name);
    }
}

fn import_teapot(app: &mut App) {
    let mut buf = Cursor::new(include_bytes!("../assets/teapot.stl"));
    app.load_mesh(&mut buf, "stl", "Utah Teapot".into());
}

fn save(app: &mut App) {
    if let Some(path) = FileDialog::new()
        .add_filter("mslicer project", &["mslicer"])
        .save_file()
    {
        if let Err(error) = app.save_project(&path) {
            error!("Error saving project: {:?}", error);
            app.popup.open(Popup::simple(
                "Error Saving Project",
                PopupIcon::Error,
                error.to_string(),
            ));
        }
    }
}

fn load(app: &mut App) {
    if let Some(path) = FileDialog::new()
        .add_filter("mslicer project", &["mslicer"])
        .pick_file()
    {
        if let Err(error) = app.load_project(&path) {
            error!("Error loading project: {:?}", error);
            app.popup.open(Popup::simple(
                "Error Loading Project",
                PopupIcon::Error,
                error.to_string(),
            ));
        }
    }
}

fn quit() {
    process::exit(0);
}
