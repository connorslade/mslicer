use common::units::Milimeter;
use egui::{Context, ViewportCommand};

use crate::{
    core::{App, project::Project},
    include_asset,
    interface::{keybinds::Keybind, popup::confirm_unsaved},
    task::{AutoLayout, FileDialog, LoadSliced, MeshLoad, MultiFileDialog},
};

pub const IMPORT_MODEL_SHORTCUT: Shortcut = Shortcut::new(Keybind::ImportModel, import_model);
pub const IMPORT_SLICED_SHORTCUT: Shortcut = Shortcut::new(Keybind::ImportSliced, load_sliced);
pub const LOAD_TEAPOT_SHORTCUT: Shortcut = Shortcut::new(Keybind::LoadTeapot, import_teapot);
pub const SAVE_PROJECT_SHORTCUT: Shortcut = Shortcut::new(Keybind::SaveProject, save);
pub const SAVE_AS_PROJECT_SHORTCUT: Shortcut = Shortcut::new(Keybind::SaveAsProject, save_as);
pub const NEW_PROJECT_SHORTCUT: Shortcut = Shortcut::new(Keybind::NewProject, new);
pub const LOAD_PROJECT_SHORTCUT: Shortcut = Shortcut::new(Keybind::LoadProject, load);
pub const QUIT_SHORTCUT: Shortcut = Shortcut::new(Keybind::Quit, quit);
pub const SLICE_SHORTCUT: Shortcut = Shortcut::new(Keybind::Slice, slice);
pub const UNDO_SHORTCUT: Shortcut = Shortcut::new(Keybind::Undo, undo);
pub const REDO_SHORTCUT: Shortcut = Shortcut::new(Keybind::Redo, redo);
pub const LAYOUT_SHORTCUT: Shortcut = Shortcut::new(Keybind::Layout, quick_layout);
pub const SELECT_ALL_SHORTCUT: Shortcut = Shortcut::new(Keybind::SelectAll, select_all);
pub const SELECT_NONE_SHORTCUT: Shortcut = Shortcut::new(Keybind::SelectNone, select_none);

const SHORTCUTS: &[Shortcut] = &[
    IMPORT_SLICED_SHORTCUT,
    IMPORT_MODEL_SHORTCUT,
    LOAD_TEAPOT_SHORTCUT,
    NEW_PROJECT_SHORTCUT,
    LOAD_PROJECT_SHORTCUT,
    SAVE_PROJECT_SHORTCUT,
    SAVE_AS_PROJECT_SHORTCUT,
    QUIT_SHORTCUT,
    UNDO_SHORTCUT,
    REDO_SHORTCUT,
    SLICE_SHORTCUT,
    LAYOUT_SHORTCUT,
    SELECT_ALL_SHORTCUT,
    SELECT_NONE_SHORTCUT,
];

type ShortcutCallback = fn(&mut App, &Context);

#[derive(Clone, Copy)]
pub struct Shortcut {
    pub keybind: Keybind,
    pub callback: ShortcutCallback,
}

impl Shortcut {
    const fn new(keybind: Keybind, callback: ShortcutCallback) -> Self {
        Self { keybind, callback }
    }
}

pub fn handle(app: &mut App, ctx: &Context) {
    for shortcut in SHORTCUTS {
        if app.config.keybinds.consume(ctx, shortcut.keybind) {
            (shortcut.callback)(app, ctx);
            break;
        }
    }
}

fn import_model(app: &mut App, _ctx: &Context) {
    app.tasks.add(MultiFileDialog::pick_files(
        ("Mesh", &["stl", "obj"]),
        |_app, paths, tasks| {
            for path in paths {
                let name = path.file_name().unwrap().to_str().unwrap().to_string();
                let ext = path.extension();
                let format = ext.unwrap_or_default().to_string_lossy();

                let task = MeshLoad::file(path.to_path_buf(), name, format.into()).unwrap();
                tasks.push(Box::new(task));
            }
        },
    ));
}

fn load_sliced(app: &mut App, _ctx: &Context) {
    app.tasks.add(FileDialog::pick_file(
        ("Sliced Model", &["goo", "ctb", "nanodlp"]),
        |_app, file, tasks| tasks.push(Box::new(LoadSliced::new(file.to_path_buf()))),
    ));
}

fn import_teapot(app: &mut App, _ctx: &Context) {
    app.tasks.add(MeshLoad::buffer(
        include_asset!("teapot.stl"),
        "Utah Teapot".into(),
        "stl".into(),
    ));
}

fn new(app: &mut App, _ctx: &Context) {
    confirm_unsaved(app, |app| {
        app.project.reset(&app.config.default_slice_config);
        app.history.clear();
        Vec::new()
    });
}

fn save(app: &mut App, _ctx: &Context) {
    let task = app.project.save(|_| Vec::new());
    app.tasks().add_boxed(task);
}

fn save_as(app: &mut App, _ctx: &Context) {
    let task = app.project.save_as(|_| Vec::new());
    app.tasks().add(task);
}

fn load(app: &mut App, _ctx: &Context) {
    confirm_unsaved(app, |_app| vec![Box::new(Project::load())]);
}

fn quit(_app: &mut App, ctx: &Context) {
    ctx.send_viewport_cmd(ViewportCommand::Close);
}

fn slice(app: &mut App, _ctx: &Context) {
    app.slice();
}

fn undo(app: &mut App, _ctx: &Context) {
    app.history().undo();
}

fn redo(app: &mut App, _ctx: &Context) {
    app.history().redo();
}

fn select_none(app: &mut App, _ctx: &Context) {
    app.state.selected.clear();
}

fn quick_layout(app: &mut App, _ctx: &Context) {
    if app.project.models.is_empty() {
        return;
    }

    let padding = app.config.ui.quick_layout_spacing.get::<Milimeter>();
    app.tasks.add(AutoLayout::new(
        &app.project.slice_config,
        &app.project.models,
        (padding, 10.0),
    ));
}

fn select_all(app: &mut App, _ctx: &Context) {
    app.state.selected.clear();
    for model in app.project.models.iter() {
        app.state.selected.select_model(model.id);
    }
}
