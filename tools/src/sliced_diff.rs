use std::{path::PathBuf, sync::Arc};

use common::{
    container::{Run, rle::downsample::RunQueue},
    progress::Progress,
    slice::{Layer, SliceConfig, SlicedFile},
};
use nalgebra::Vector2;

#[derive(Clone)]
pub struct SlicedDiff {
    pub old: Source,
    pub new: Source,

    pub difference: Difference,
    pub threshold: (bool, u8),
}

#[derive(Clone, Default)]
pub enum Source {
    #[default]
    Empty,
    File {
        path: PathBuf,
        file: Arc<Box<dyn SlicedFile + Send + Sync>>,
    },
    Loaded {
        config: SliceConfig,
        layers: Arc<Vec<Layer>>,
    },
}

#[derive(Clone, Copy)]
pub enum SourceId {
    Old,
    New,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Difference {
    Absolute,
    Signed,
}

impl SlicedDiff {
    pub fn slice_config(&self, _config: &mut SliceConfig) {}

    pub fn generate(&self, config: &SliceConfig, progress: &Progress) -> Vec<Layer> {
        let layers = self.old.layers().max(self.new.layers());
        progress.set_total((self.old.layers() + self.new.layers() + layers) as u64);

        let (loaded_config, old) = self.old.load(progress).unwrap();
        let (_, new) = self.new.load(progress).unwrap();

        let platform = loaded_config.platform_resolution;
        let pixels = platform.x as u64 * platform.y as u64;
        let empty = Arc::new(vec![Run::new(pixels, 0)]);

        let layers = (0..layers)
            .map(|i| {
                let [old, new] = [&old, &new].map(|x| x.get(i).map(|x| &x.data).unwrap_or(&empty));
                let data = self.diff_layer(old, new);

                Layer::new(
                    data,
                    config.default_height(i as u32),
                    config.exposure_config(i as u32).into_owned(),
                )
            })
            .inspect(|_| progress.add_complete(1))
            .collect();

        progress.set_finished();
        layers
    }

    pub fn sources_specified(&self) -> bool {
        self.old.is_specified() && self.new.is_specified()
    }

    /// Checks if both sources are specified and have the same layer resolution
    pub fn matching_sources(&self) -> bool {
        self.old.resolution() == self.new.resolution()
    }

    pub fn source_mut(&mut self, id: SourceId) -> &mut Source {
        match id {
            SourceId::Old => &mut self.old,
            SourceId::New => &mut self.new,
        }
    }

    fn threshold(&self, value: u8) -> u8 {
        if self.threshold.0 {
            [0, 255][(value >= self.threshold.1) as usize]
        } else {
            value
        }
    }

    fn diff_layer(&self, old: &[Run], new: &[Run]) -> Vec<Run> {
        let (mut old, mut new) = (RunQueue::new(old), RunQueue::new(new));
        let mut out = Vec::new();

        // uncompressed length of old and new must be the same
        while old.remaining() {
            let length = old.active.length.min(new.active.length);
            let (old, new) = (old.take_up_to(length), new.take_up_to(length));

            let [old_value, new_value] = [old.value, new.value].map(|x| self.threshold(x));
            let value = match self.difference {
                Difference::Absolute => old_value.abs_diff(new_value),
                Difference::Signed => {
                    let diff = new_value as i16 - old_value as i16;
                    (diff / 2 + 128) as u8
                }
            };

            out.push(Run::new(length, value));
        }

        out
    }
}

impl Source {
    pub fn is_specified(&self) -> bool {
        !matches!(self, Self::Empty)
    }

    pub fn resolution(&self) -> Option<Vector2<u32>> {
        Some(match self {
            Source::Empty => return None,
            Source::File { file, .. } => file.slice_config().platform_resolution,
            Source::Loaded { config, .. } => config.platform_resolution,
        })
    }

    pub fn layers(&self) -> usize {
        match self {
            Source::Empty => 0,
            Source::File { file, .. } => file.layer_count(),
            Source::Loaded { layers, .. } => layers.len(),
        }
    }

    fn load(&self, progress: &Progress) -> Option<(SliceConfig, Arc<Vec<Layer>>)> {
        Some(match self {
            Source::File { file, .. } => {
                let layers = file.layers(progress);
                (file.slice_config(), Arc::new(layers))
            }
            Source::Loaded { config, layers } => (config.clone(), layers.clone()),
            Source::Empty => return None,
        })
    }
}

impl Difference {
    pub const ALL: [Self; 2] = [Self::Absolute, Self::Signed];

    pub fn name(&self) -> &str {
        match self {
            Difference::Absolute => "Absolute Difference",
            Difference::Signed => "Signed Difference",
        }
    }
}

impl Default for SlicedDiff {
    fn default() -> Self {
        Self {
            old: Default::default(),
            new: Default::default(),

            difference: Difference::Signed,
            threshold: (false, 128),
        }
    }
}
