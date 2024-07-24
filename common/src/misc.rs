use std::time::Duration;

use crate::config::SliceConfig;

pub struct SliceResult<'a, Layer> {
    pub layers: Vec<Layer>,
    pub slice_config: &'a SliceConfig,
}

pub struct Run {
    pub length: u64,
    pub value: u8,
}

pub trait EncodableLayer {
    type Output: Send;

    fn new() -> Self;
    fn add_run(&mut self, length: u64, value: u8);
    fn finish(self, layer: usize, config: &SliceConfig) -> Self::Output;
}

pub fn human_duration(duration: Duration) -> String {
    let ms = duration.as_millis() as f32;
    if ms < 1000.0 {
        format!("{:}ms", ms)
    } else if ms < 60_000.0 {
        format!("{:.2}s", ms / 1000.0)
    } else {
        let minutes = ms / 60_000.0;
        let seconds = (minutes - minutes.floor()) * 60.0;
        format!("{:.0}m {:.2}s", minutes.floor(), seconds)
    }
}
