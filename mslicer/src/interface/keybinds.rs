use egui::{Key, KeyboardShortcut, ModifierNames, Modifiers, Ui};
use serde::{Deserialize, Serialize};

use crate::interface::components::grid;

#[derive(Clone, Serialize, Deserialize)]
pub struct Keybinds {
    tool_pointer: KeyboardShortcut,
    tool_support: KeyboardShortcut,
    tool_orient: KeyboardShortcut,
}

pub enum Keybind {
    ToolSelect,
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
            Keybind::ToolSelect => self.tool_pointer,
            Keybind::ToolSupport => self.tool_support,
            Keybind::ToolOrient => self.tool_orient,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        ui.collapsing("Tools", |ui| {
            grid("tools").show(ui, |ui| {
                ui.label("Pointer");
                ui.horizontal(|ui| {
                    shortcut_widget(ui, &mut self.tool_pointer);
                    ui.take_available_width();
                });
                ui.end_row();

                ui.label("Support");
                shortcut_widget(ui, &mut self.tool_support);
                ui.end_row();

                ui.label("Orient");
                shortcut_widget(ui, &mut self.tool_orient);
                ui.end_row();
            });
        });
    }
}

fn shortcut_widget(ui: &mut Ui, shortcut: &mut KeyboardShortcut) {
    if ui
        .button(shortcut.format(&ModifierNames::NAMES, cfg!(target_os = "macos")))
        .clicked()
    {}
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            tool_pointer: KeyboardShortcut::new(Modifiers::NONE, Key::Num1),
            tool_support: KeyboardShortcut::new(Modifiers::NONE, Key::Num2),
            tool_orient: KeyboardShortcut::new(Modifiers::NONE, Key::Num3),
        }
    }
}
