use std::mem;

use egui::Vec2;
use egui_dock::{DockState, NodeIndex, Tree};
use nalgebra::Vector2;

use crate::{app::config::Config, windows::Tab};

pub struct Panels {
    pub dock_state: DockState<Tab>,
}

impl Panels {
    pub fn new(config: &mut Config) -> Self {
        let mut dock_state = DockState::new(vec![Tab::Viewport]);
        let surface = dock_state.main_surface_mut();

        if let Some(past_state) = &mut config.panels {
            *surface = mem::take(past_state);
        } else {
            default_dock_layout(surface);
        }

        match surface.find_tab(&Tab::Viewport) {
            Some((ni, ti)) => surface.set_active_tab(ni, ti),
            None => *surface = Tree::new(vec![Tab::Viewport]),
        }

        Self { dock_state }
    }

    pub fn focus_tab(&mut self, tab: Tab, size: Vector2<f32>) {
        if let Some(panel) = self.dock_state.find_tab(&tab) {
            self.dock_state.set_active_tab(panel);
        } else {
            self.add_tab(tab, size);
        }
    }

    pub fn add_tab(&mut self, tab: Tab, size: Vector2<f32>) {
        let window_id = self.dock_state.add_window(vec![tab]);
        let window = self.dock_state.get_window_state_mut(window_id).unwrap();
        window.set_size(Vec2::new(size.x, size.y));
    }

    pub fn reset_ui(&mut self) {
        self.dock_state = DockState::new(vec![Tab::Viewport]);
        let surface = self.dock_state.main_surface_mut();
        default_dock_layout(surface);
    }

    pub fn checkbox(&mut self, tab: Tab, callback: impl FnOnce(&mut bool)) {
        let existing = self.dock_state.find_tab(&tab);
        let mut open = existing.is_some();
        callback(&mut open);

        if !open && let Some(tab) = existing {
            self.dock_state.remove_tab(tab);
        } else if open && existing.is_none() {
            self.dock_state.add_window(vec![tab]);
        }
    }
}

fn default_dock_layout(surface: &mut Tree<Tab>) {
    let [_old_node, new_node] = surface.split_left(NodeIndex::root(), 0.2, vec![Tab::Models]);
    let [_old_node, new_node] =
        surface.split_below(new_node, 0.4, vec![Tab::SliceConfig, Tab::Supports]);
    surface.split_below(new_node, 0.6, vec![Tab::Workspace, Tab::RemotePrint]);
}
