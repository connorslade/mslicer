use std::path::PathBuf;

use crate::app::project::model::Model;
use common::{
    progress::CombinedProgress,
    slice::{DynSlicedFile, SliceConfig},
};
use slicer::post_process::{anti_alias::AntiAlias, elephant_foot_fixer::ElephantFootFixer};

pub mod model;
pub mod storage;

#[derive(Default, Clone)]
pub struct Project {
    pub path: Option<PathBuf>,
    pub slice_config: SliceConfig,
    pub post_processing: PostProcessing,
    pub models: Vec<Model>,
}

#[derive(Default, Clone)]
pub struct PostProcessing {
    pub anti_alias: AntiAlias,
    pub elephant_foot_fixer: ElephantFootFixer,
}

impl Project {
    pub fn with_path(self, path: PathBuf) -> Self {
        Self {
            path: Some(path),
            ..self
        }
    }
}

impl PostProcessing {
    pub fn process(&self, file: &mut DynSlicedFile, progress: CombinedProgress<2>) {
        self.elephant_foot_fixer
            .post_slice(file, progress[0].clone());
        self.anti_alias.post_slice(file, progress[1].clone());
    }
}
