use std::{fs::File, path::PathBuf};

use anyhow::Result;
use common::{
    container::{Run, rle::downsample::RunQueue},
    progress::{CombinedProgress, Progress},
    slice::{Layer, SliceConfig, format::RasterFormat},
};
use slicer::util::load_sliced;

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
    File(PathBuf),
    Loaded,
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
        let _progress = CombinedProgress::<3>::new();
        let old = self.old.load(&_progress[0]).unwrap();
        let new = self.new.load(&_progress[1]).unwrap();

        let layers = old.len().min(new.len()); // todo: max
        progress.set_total(layers as u64);
        (0..layers)
            .map(|i| {
                let (old, new) = (&old[i].data, &new[i].data);
                let data = self.diff_layer(old, new);

                Layer::new(
                    data,
                    config.default_height(i as u32),
                    config.exposure_config(i as u32).into_owned(),
                )
            })
            .inspect(|_| progress.add_complete(1))
            .collect()
    }

    pub fn sources_specified(&self) -> bool {
        self.old.is_specified() && self.new.is_specified()
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

    fn load(&self, progress: &Progress) -> Result<Vec<Layer>> {
        Ok(match self {
            Source::File(path) => {
                let format =
                    RasterFormat::from_extension(&path.extension().unwrap().to_string_lossy())
                        .unwrap();
                let file = File::open(path)?;
                load_sliced(progress, &format, file)?.1
            }
            Source::Loaded => todo!(),
            Source::Empty => unreachable!(),
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
