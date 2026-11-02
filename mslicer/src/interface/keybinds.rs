use egui::{Context, Event, Key, KeyboardShortcut, ModifierNames, Modifiers, Ui};
use serde::{Deserialize, Serialize};

use crate::interface::components::grid;

macro_rules! keybinds {
    ($(($field:ident, $keybind:ident, $name:expr) => ($modif:expr, $key:expr)),*) => {
        #[derive(Clone, Serialize, Deserialize)]
        pub struct Keybinds {
            #[serde(skip)]
            editing: Option<Keybind>,

            $($field: KeyboardShortcut),*
        }

        #[derive(Clone, Copy, PartialEq, Eq)]
        pub enum Keybind {
            $($keybind),*
        }

        impl Keybinds {
            pub fn get(&self, key: Keybind) -> KeyboardShortcut {
                match key {
                    $(Keybind::$keybind => self.$field),*
                }
            }

            pub fn get_mut(&mut self, key: Keybind) -> &mut KeyboardShortcut {
                match key {
                    $(Keybind::$keybind => &mut self.$field),*
                }
            }
        }

        impl Keybind {
            pub fn name(&self) -> &str {
                match self {
                    $(Self::$keybind => $name),*
                }
            }
        }

        impl Default for Keybinds {
            fn default() -> Self {
                Self {
                    editing: None,

                    $($field: KeyboardShortcut::new($modif, $key)),*
                }
            }
        }

    };
}

const COMMAND_SHIFT: Modifiers = Modifiers::COMMAND.plus(Modifiers::SHIFT);

keybinds![
    (pointer, Pointer, "Pointer") => (Modifiers::NONE, Key::Num1),
    (support, Support, "Support") => (Modifiers::NONE, Key::Num2),
    (orient, Orient, "Orient") => (Modifiers::NONE, Key::Num3),

    (delete, Delete, "Delete") => (Modifiers::NONE, Key::D),
    (collect, Collect, "Collect") => (Modifiers::NONE, Key::G),
    (duplicate, Duplicate, "Duplicate") => (Modifiers::NONE, Key::C),
    (rename, Rename, "Rename") => (Modifiers::NONE, Key::R),
    (hide, Hide, "Hide") => (Modifiers::NONE, Key::H),
    (select, Select, "Select") => (Modifiers::NONE, Key::S),
    (align, Align, "Align") => (Modifiers::NONE, Key::A),
    (split, Split, "Split") => (Modifiers::NONE, Key::F),

    (import_model, ImportModel, "Import Model") => (Modifiers::COMMAND, Key::I),
    (import_sliced, ImportSliced, "Import Sliced") => (COMMAND_SHIFT, Key::I),
    (load_teapot, LoadTeapot, "Load Teapot") => (Modifiers::COMMAND, Key::T),
    (save_project, SaveProject, "Save Project") => (Modifiers::COMMAND, Key::S),
    (save_as_project, SaveAsProject, "Save As Project") => (COMMAND_SHIFT, Key::S),
    (new_project, NewProject, "New Project") => (Modifiers::COMMAND, Key::N),
    (load_project, LoadProject, "Load Project") => (Modifiers::COMMAND, Key::O),
    (quit, Quit, "Quit") => (Modifiers::COMMAND, Key::Q),
    (slice, Slice, "Slice") => (Modifiers::COMMAND, Key::R),
    (undo, Undo, "Undo") => (Modifiers::COMMAND, Key::Z),
    (redo, Redo, "Redo") => (Modifiers::COMMAND, Key::Y),
    (layout, Layout, "Layout") => (Modifiers::COMMAND, Key::L),
    (select_all, SelectAll, "Select All") => (Modifiers::COMMAND, Key::A),
    (select_none, SelectNone, "Select None") => (COMMAND_SHIFT, Key::D)
];

impl Keybinds {
    pub fn consume(&self, ctx: &Context, key: Keybind) -> bool {
        let shortcut = self.get(key);
        !ctx.wants_keyboard_input() && ctx.input_mut(|i| i.consume_shortcut(&shortcut))
    }

    pub fn update(&mut self, ctx: &Context) {
        let Some(editing) = self.editing else { return };

        ctx.input_mut(|i| {
            for (idx, event) in i.events.iter().enumerate() {
                if let Event::Key {
                    key,
                    modifiers,
                    pressed: true,
                    ..
                } = event
                {
                    let shortcut = self.get_mut(editing);
                    *shortcut = KeyboardShortcut::new(*modifiers, *key);
                    self.editing = None;
                    i.events.remove(idx);
                    break;
                }
            }
        });
    }

    fn shortcut_row(&mut self, ui: &mut Ui, name: &str, key: Keybind) {
        ui.label(name);
        ui.horizontal(|ui| {
            let shortcut = self.get(key);
            if self.editing == Some(key) {
                if ui.button("…").highlight().clicked() {
                    self.editing = None;
                }
            } else {
                if ui
                    .button(shortcut.format(&ModifierNames::NAMES, cfg!(target_os = "macos")))
                    .clicked()
                {
                    self.editing = Some(key);
                }
            };

            ui.take_available_width();
        });
        ui.end_row();
    }

    fn shortcut_section(&mut self, ui: &mut Ui, name: &str, binds: &[Keybind]) {
        ui.collapsing(name, |ui| {
            grid(name).show(ui, |ui| {
                for bind in binds {
                    self.shortcut_row(ui, bind.name(), *bind);
                }
            });
        });
    }

    #[rustfmt::skip]
    pub fn ui(&mut self, ui: &mut Ui) {
        use Keybind::*;

        self.shortcut_section(ui, "General", &[ImportModel, ImportSliced,
            LoadTeapot, SaveProject, SaveAsProject, NewProject, LoadProject,
            Quit, Slice, Undo, Redo, Layout, SelectAll, SelectNone]);
        self.shortcut_section(ui, "Tools", &[Pointer, Support, Orient]);
        self.shortcut_section(ui, "Selection", &[Delete, Collect, Duplicate,
            Rename, Hide, Select, Align, Split]);
    }
}
