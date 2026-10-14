use std::sync::Arc;

use common::{
    container::Run,
    progress::Progress,
    slice::{Layer, SliceConfig},
};

#[derive(Clone)]
pub struct TestPattern {
    pub pattern: Pattern,
    pub layers: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Pattern {
    Empty,
    Solid,
    Checkerboard,
}

impl TestPattern {
    pub fn slice_config(&self, _config: &mut SliceConfig) {}

    pub fn generate(&self, config: &SliceConfig, progress: &Progress) -> Vec<Layer> {
        let pixels = config.platform_resolution.x as u64 * config.platform_resolution.y as u64;
        progress.set_total(1 + self.layers as u64);

        let mut data = Vec::new();
        match self.pattern {
            Pattern::Empty => data.push(Run::new(pixels, 0)),
            Pattern::Solid => data.push(Run::new(pixels, 255)),
            Pattern::Checkerboard => {
                for y in 0..config.platform_resolution.y {
                    let y = y % 2;
                    for x in 0..config.platform_resolution.x {
                        let value = x % 2 == y;
                        data.push(Run::new(1, [0, 255][value as usize]));
                    }
                }
            }
        }
        let data = Arc::new(data);
        progress.add_complete(1);

        let mut out = Vec::new();
        for i in 0..self.layers {
            out.push(Layer::new_arc(
                data.clone(),
                config.default_height(i),
                config.exposure_config(i).into_owned(),
            ));
            progress.add_complete(1);
        }

        progress.set_finished();
        out
    }
}

impl Pattern {
    pub const ALL: [Self; 3] = [Self::Empty, Self::Solid, Self::Checkerboard];

    pub fn name(&self) -> &str {
        match self {
            Pattern::Empty => "Empty",
            Pattern::Solid => "Solid",
            Pattern::Checkerboard => "Checkerboard",
        }
    }
}

impl Default for TestPattern {
    fn default() -> Self {
        Self {
            pattern: Pattern::Solid,
            layers: 1,
        }
    }
}
