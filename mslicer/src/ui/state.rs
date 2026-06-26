use std::{collections::HashSet, iter};

use egui::Vec2;
use egui_tracing::EventCollector;
use itertools::Either;
use nalgebra::{Vector2, Vector3};
use slicer::mesh::Mesh;
use tools::supports::SupportConfig;

use crate::windows::tools::Tools;

#[derive(Default)]
pub struct UiState {
    pub event_collector: EventCollector,
    pub support_config: SupportConfig,
    pub line_support_debug: Vec<[Vector3<f32>; 2]>,
    pub queue_reset_ui: bool,

    // support stuff
    pub workspace: WorkspaceHover,
    pub support_placement: bool,

    pub selected: Selected,
    pub selected_printer: usize,
    pub support_preview: Option<Mesh>,

    pub selected_remap_point: Option<u8>,

    // remote send ui
    pub working_address: String,
    pub working_filename: String,
    pub remote_print_connecting: RemotePrintConnectStatus,

    // slice preview
    pub slice_preview_layer: usize,
    pub last_preview_layer: usize,
    pub preview_offset: Vector2<f32>,
    pub preview_scale: f32,
    pub layer_count: (usize, u8),

    // tools
    pub tools: Tools,

    pub move_timeout: u32,
}

#[derive(Default)]
pub enum Selected {
    #[default]
    None,
    Models(HashSet<u32>),
    Collection(u32),
}

#[derive(Default)]
pub struct WorkspaceHover {
    pub is_moving: bool,
    pub aspect: f32,
    pub uv: Vector2<f32>,
}

#[derive(Default, PartialEq, Eq)]
pub enum RemotePrintConnectStatus {
    #[default]
    None,
    Connecting,
    Scanning,
}

impl Selected {
    pub fn clear(&mut self) {
        *self = Selected::None;
    }

    pub fn model_clicked(&mut self, id: u32, shift: bool) {
        match self {
            Selected::None | Selected::Collection(_) => {
                let mut set = HashSet::new();
                set.insert(id);
                *self = Selected::Models(set);
            }
            Selected::Models(set) => {
                if shift {
                    if set.contains(&id) {
                        set.remove(&id);
                    } else {
                        set.insert(id);
                    }
                } else {
                    set.clear();
                    set.insert(id);
                }
            }
        }
    }

    pub fn selected_models(&self) -> impl Iterator<Item = u32> {
        match self {
            Selected::None | Selected::Collection(_) => Either::Left(iter::empty()),
            Selected::Models(set) => Either::Right(set.iter().copied()),
        }
    }

    pub fn contains_model(&self, id: u32) -> bool {
        match self {
            Selected::Models(set) => set.contains(&id),
            _ => false,
        }
    }

    pub fn single_model(&self) -> Option<u32> {
        match self {
            Selected::Models(set) if set.len() == 1 => set.iter().next().copied(),
            _ => None,
        }
    }

    pub fn has_models(&self) -> bool {
        match self {
            Selected::Models(set) => set.len() > 0,
            _ => false,
        }
    }

    pub fn contains_collection(&self, id: u32) -> bool {
        match self {
            Selected::Collection(collection) => *collection == id,
            _ => false,
        }
    }

    pub fn collection_clicked(&mut self, id: u32, shift: bool) {
        match self {
            Selected::Models(set) if !shift || set.is_empty() => *self = Self::Collection(id),
            Selected::Collection(group) if id == *group => self.clear(),
            Selected::Collection(_) | Selected::None => *self = Self::Collection(id),
            _ => {}
        }
    }
}

impl WorkspaceHover {
    pub fn new(is_moving: bool, aspect: f32, uv: Vec2) -> Self {
        Self {
            is_moving,
            aspect,
            uv: Vector2::new(uv.x, uv.y),
        }
    }

    pub fn hovered(&self) -> bool {
        self.uv.x >= 0.0 && self.uv.y >= 0.0
    }
}
