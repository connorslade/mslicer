use crate::{config::SliceConfig, image::Image};

pub struct SliceResult<'a> {
    pub layers: Vec<Image>,
    pub slice_config: &'a SliceConfig,
}

pub struct Run {
    pub length: u64,
    pub value: u8,
}
