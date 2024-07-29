use egui_tracing::EventCollector;

#[derive(Default)]
pub struct UiState {
    pub event_collector: EventCollector,

    // remote send ui
    pub working_address: String,
    pub working_filename: String,
    pub send_print_completion: bool,
    pub remote_print_connecting: RemotePrintConnectStatus,
}

#[derive(Default, PartialEq, Eq)]
pub enum RemotePrintConnectStatus {
    #[default]
    None,
    Connecting,
    Scanning,
}
