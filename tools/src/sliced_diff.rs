use std::{fs::File, path::PathBuf};

use anyhow::Result;
use common::{
    container::{Run, rle::downsample::RunQueue},
    progress::{CombinedProgress, Progress},
    slice::{Layer, SliceConfig, format::RasterFormat},
};
use slicer::util::load_sliced;

// grey means voxel is the same in both, black or white is the voxel value of new

#[derive(Clone)]
pub struct SlicedDiff {
    pub old: Source,
    pub new: Source,
    pub threshold: u8,
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
                let data = diff_layer(old, new, self.threshold);

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

fn diff_layer(old: &[Run], new: &[Run], threshold: u8) -> Vec<Run> {
    let (mut old, mut new) = (RunQueue::new(old), RunQueue::new(new));
    let mut out = Vec::new();

    // uncompressed length of old and new must be the same
    while old.remaining() {
        let length = old.active.length.min(new.active.length);
        let (old, new) = (old.take_up_to(length), new.take_up_to(length));

        let new_threshold = new.value >= threshold;
        if (old.value >= threshold) ^ new_threshold {
            out.push(Run::new(length, new_threshold as u8 * 255));
        } else {
            out.push(Run::new(length, 128));
        }
    }

    out
}

impl Default for SlicedDiff {
    fn default() -> Self {
        Self {
            old: Default::default(),
            new: Default::default(),
            threshold: 128,
        }
    }
}
