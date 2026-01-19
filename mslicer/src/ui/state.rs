use egui::Vec2;
use egui_tracing::EventCollector;
use nalgebra::{Vector2, Vector3};
use slicer::{mesh::Mesh, supports::line::LineSupportConfig};

use super::markdown::CompiledMarkdown;

#[derive(Default)]
pub struct UiState {
    pub event_collector: EventCollector,
    pub line_support_config: LineSupportConfig,
    pub line_support_debug: Vec<[Vector3<f32>; 2]>,
    pub queue_reset_ui: bool,

    // support stuff
    pub workspace: WorkspaceHover,
    pub support_placement: bool,

    pub selected_printer: usize,
    pub support_preview: Option<Mesh>,

    // remote send ui
    pub working_address: String,
    pub working_filename: String,
    pub send_print_completion: bool,
    pub remote_print_connecting: RemotePrintConnectStatus,

    // documentation
    pub docs_page: DocsPage,
    pub compiled_markdown: CompiledMarkdown,
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

#[derive(Default, Clone, Copy, PartialEq)]
pub enum DocsPage {
    #[default]
    GettingStarted,
    Miscellaneous,
}

impl WorkspaceHover {
    pub fn new(is_moving: bool, aspect: f32, uv: Vec2) -> Self {
        Self {
            is_moving,
            aspect,
            uv: Vector2::new(uv.x, uv.y),
        }
    }
}

impl DocsPage {
    pub const ALL: [DocsPage; 2] = [DocsPage::GettingStarted, DocsPage::Miscellaneous];

    pub fn name(&self) -> &'static str {
        match self {
            DocsPage::GettingStarted => "Getting Started",
            DocsPage::Miscellaneous => "Miscellaneous",
        }
    }

    pub fn source(&self) -> &'static str {
        match self {
            DocsPage::GettingStarted => include_str!("../../../docs/getting_started.md"),
            DocsPage::Miscellaneous => include_str!("../../../docs/miscellaneous.md"),
        }
    }
}
