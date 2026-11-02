use egui::{Context, Event, Key, KeyboardShortcut, ModifierNames, Modifiers, Ui};
use serde::{Deserialize, Serialize};

use crate::interface::components::grid;

#[derive(Clone, Serialize, Deserialize)]
pub struct Keybinds {
    #[serde(skip)]
    editing: Option<Keybind>,

    tool_pointer: KeyboardShortcut,
    tool_support: KeyboardShortcut,
    tool_orient: KeyboardShortcut,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Keybind {
    ToolPointer,
    ToolSupport,
    ToolOrient,
}

impl Keybinds {
    pub fn consume(&self, ui: &Ui, key: Keybind) -> bool {
        let shortcut = self.get(key);
        !ui.ctx().wants_keyboard_input() && ui.input_mut(|i| i.consume_shortcut(&shortcut))
    }

    pub fn get(&self, key: Keybind) -> KeyboardShortcut {
        match key {
            Keybind::ToolPointer => self.tool_pointer,
            Keybind::ToolSupport => self.tool_support,
            Keybind::ToolOrient => self.tool_orient,
        }
    }

    pub fn get_mut(&mut self, key: Keybind) -> &mut KeyboardShortcut {
        match key {
            Keybind::ToolPointer => &mut self.tool_pointer,
            Keybind::ToolSupport => &mut self.tool_support,
            Keybind::ToolOrient => &mut self.tool_orient,
        }
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

    pub fn ui(&mut self, ui: &mut Ui) {
        ui.collapsing("Tools", |ui| {
            grid("tools").show(ui, |ui| {
                self.shortcut_row(ui, "Pointer", Keybind::ToolPointer);
                self.shortcut_row(ui, "Support", Keybind::ToolSupport);
                self.shortcut_row(ui, "Orient", Keybind::ToolOrient);
            })
        });
    }
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            editing: None,

            tool_pointer: KeyboardShortcut::new(Modifiers::NONE, Key::Num1),
            tool_support: KeyboardShortcut::new(Modifiers::NONE, Key::Num2),
            tool_orient: KeyboardShortcut::new(Modifiers::NONE, Key::Num3),
        }
    }
}
