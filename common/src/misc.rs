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
