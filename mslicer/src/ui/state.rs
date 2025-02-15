use egui_tracing::EventCollector;
use nalgebra::Vector3;
use slicer::supports::line::LineSupportConfig;

#[derive(Default)]
pub struct UiState {
    pub event_collector: EventCollector,
    pub line_support_config: LineSupportConfig,
    pub line_support_debug: Vec<[Vector3<f32>; 2]>,

    // remote send ui
    pub working_address: String,
    pub working_filename: String,
    pub send_print_completion: bool,
    pub remote_print_connecting: RemotePrintConnectStatus,

    // documentation
    pub docs_page: DocsPage,
}

#[derive(Default, PartialEq, Eq)]
pub enum RemotePrintConnectStatus {
    #[default]
    None,
    Connecting,
    Scanning,
}

#[derive(Default, PartialEq)]
pub enum DocsPage {
    #[default]
    GettingStarted,
    AnotherPage,
}
