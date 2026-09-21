use std::sync::Arc;

use egui::Vec2;
use egui_tracing::EventCollector;
use nalgebra::{Vector2, Vector3};
use slicer::mesh::Mesh;

use crate::{
    core::{
        config::peripherals::Webhook,
        project::model::ModelId,
        selected::{SelectedModel, SelectedPrinter, SelectedSupports},
    },
    interface::tools::Tools,
};

#[derive(Default)]
pub struct UiState {
    pub event_collector: EventCollector,
    pub line_support_debug: Vec<[Vector3<f32>; 2]>,
    pub queue_reset_ui: bool,

    // support stuff
    pub workspace: WorkspaceHover,
    pub hovered_geometry: Option<GeometryHit>,
    pub support_placement: bool,
    pub support_mode: bool,

    pub selected: SelectedModel,
    pub selected_printer: SelectedPrinter,
    pub selected_supports: SelectedSupports,
    pub support_preview: Option<Mesh>,

    pub selected_remap_point: Option<u8>,

    // remote send ui
    pub working_address: String,
    pub working_filename: String,
    pub remote_print_connecting: RemotePrintConnectStatus,
    pub shared_webhook: Arc<SharedPrintCompletion>,

    // slice preview
    pub preview_layer: usize,
    pub last_preview_layer: usize,
    pub preview_offset: Vector2<f32>,
    pub preview_scale: f32,
    pub layer_count: (usize, u8),

    pub anisotropic_aa: bool,

    pub tools: Tools,
    pub move_timeout: u32,
}

#[derive(Default)]
pub struct SharedPrintCompletion {
    pub webhook: Webhook,
    pub alert: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct GeometryHit {
    pub model: ModelId,
    pub support: bool,

    pub face: u32,
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
