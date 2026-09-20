use std::{
    mem,
    ops::{Deref, DerefMut},
};

use egui::Vec2;

mod logs;
mod models;
pub mod remote_print;
mod slice_config;
mod sliced;
mod supports;
mod tabs;
mod viewport;
mod workspace;

use egui_dock::{DockState, Node, NodeIndex, Tree};
use nalgebra::Vector2;

use crate::core::config::Config;
pub use tabs::{Tab, Tabs};

pub struct Panels {
    pub dock_state: Option<DockState<Tab>>,
}

impl Panels {
    pub fn new(config: &mut Config) -> Self {
        let mut dock_state = DockState::new(vec![Tab::Viewport]);
        let surface = dock_state.main_surface_mut();

        if let Some(past_state) = &mut config.ui.panels {
            *surface = mem::take(past_state);
        } else {
            default_dock_layout(surface);
        }

        match surface.find_tab(&Tab::Viewport) {
            Some((ni, ti)) => surface.set_active_tab(ni, ti),
            None => *surface = Tree::new(vec![Tab::Viewport]),
        }

        Self {
            dock_state: Some(dock_state),
        }
    }

    pub fn take_inner(&mut self) -> DockState<Tab> {
        self.dock_state.take().unwrap()
    }

    pub fn replace_inner(&mut self, state: DockState<Tab>) {
        (self.dock_state.is_none()).then(|| self.dock_state = Some(state));
    }

    pub fn focus_tab(&mut self, tab: Tab, size: Vector2<f32>) {
        if let Some(panel) = self.find_tab(&tab) {
            self.set_active_tab(panel);
        } else {
            self.add_tab(tab, size);
        }
    }

    pub fn add_tab(&mut self, tab: Tab, size: Vector2<f32>) {
        let window_id = self.add_window(vec![tab]);
        let window = self.get_window_state_mut(window_id).unwrap();
        window.set_size(Vec2::new(size.x, size.y));
    }

    pub fn reset_ui(&mut self) {
        self.dock_state = Some(DockState::new(vec![Tab::Viewport, Tab::Sliced]));
        let surface = self.main_surface_mut();
        default_dock_layout(surface);
    }

    pub fn checkbox(&mut self, tab: Tab, callback: impl FnOnce(&mut bool)) {
        let existing = self.find_tab(&tab);
        let mut open = existing.is_some();
        callback(&mut open);

        if !open && let Some(tab) = existing {
            self.remove_tab(tab);
        } else if open && existing.is_none() {
            self.add_window(vec![tab]);
        }
    }

    pub fn update(&mut self, window_width: f32) {
        let surface = self.main_surface_mut();
        let (mut node, _) = surface.find_tab(&Tab::Viewport).unwrap();

        while let Some(parent) = node.parent()
            && parent != NodeIndex::root()
        {
            node = parent;
        }

        let Node::Horizontal(split) = &mut surface[NodeIndex::root()] else {
            return;
        };

        let previous_width = split.rect.width();
        let min_size = 325.0 / window_width;
        let change = previous_width / window_width;
        split.fraction = if node.is_left() {
            1.0 - (1.0 - split.fraction) * change
        } else {
            split.fraction * change
        }
        .clamp(min_size, 1.0);
    }
}

fn default_dock_layout(surface: &mut Tree<Tab>) {
    let [_old_node, new_node] = surface.split_left(NodeIndex::root(), 0.2, vec![Tab::Models]);
    let [_old_node, new_node] =
        surface.split_below(new_node, 0.4, vec![Tab::SliceConfig, Tab::Supports]);
    surface.split_below(new_node, 0.6, vec![Tab::Workspace, Tab::RemotePrint]);
}

impl Deref for Panels {
    type Target = DockState<Tab>;

    fn deref(&self) -> &Self::Target {
        self.dock_state.as_ref().unwrap()
    }
}

impl DerefMut for Panels {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.dock_state.as_mut().unwrap()
    }
}
