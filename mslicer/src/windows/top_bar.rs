use std::{
    fs::File,
    io::{BufReader, Cursor},
    process,
};

use const_format::concatcp;
use egui::{Button, Context, Key, KeyboardShortcut, Modifiers, TopBottomPanel};
use egui_phosphor::regular::STACK;
use rfd::FileDialog;

use crate::app::App;

const IMPORT_MODEL_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::I);
const LOAD_TEAPOT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::T);
const QUIT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Q);

pub fn ui(app: &mut App, ctx: &Context) {
    ctx.input_mut(|x| x.consume_shortcut(&IMPORT_MODEL_SHORTCUT))
        .then(|| import_model(app));
    ctx.input_mut(|x| x.consume_shortcut(&LOAD_TEAPOT_SHORTCUT))
        .then(|| import_teapot(app));
    ctx.input_mut(|x| x.consume_shortcut(&QUIT_SHORTCUT))
        .then(|| quit());

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

                let _ = ui.button("Save Project");
                let _ = ui.button("Load Project");

                ui.separator();

                let quit_button =
                    ui.add(Button::new("Quit").shortcut_text(ctx.format_shortcut(&QUIT_SHORTCUT)));
                quit_button.clicked().then(|| quit());
            });

            let slicing = match &app.slice_operation {
                Some(operation) => operation.progress.completed() < operation.progress.total(),
                None => false,
            };
            ui.add_enabled_ui(!slicing, |ui| {
                ui.button(concatcp!(STACK, " Slice"))
                    .clicked()
                    .then(|| app.slice());
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

fn quit() {
    process::exit(0);
}
