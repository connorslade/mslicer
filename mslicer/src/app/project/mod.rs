use std::{
    iter,
    path::{Path, PathBuf},
};

use itertools::Itertools;

use crate::app::{
    App,
    project::model::Model,
    task::{ProjectLoad, ProjectSave},
};
use common::{config::SliceConfig, progress::CombinedProgress};
use slicer::{
    format::FormatSliceFile,
    post_process::{anti_alias::AntiAlias, elephant_foot_fixer::ElephantFootFixer},
};

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

impl App {
    fn add_recent_project(&mut self, path: PathBuf) {
        self.config.recent_projects = iter::once(path)
            .chain(self.config.recent_projects.iter().cloned())
            .unique()
            .take(5)
            .collect()
    }

    pub fn save_project(&mut self, path: &Path) {
        self.add_recent_project(path.to_path_buf());
        self.tasks
            .add(ProjectSave::new(self.project.clone(), path.to_path_buf()));
    }

    pub fn load_project(&mut self, path: &Path) {
        self.add_recent_project(path.to_path_buf());
        self.tasks.add(ProjectLoad::new(path.to_path_buf()));
    }
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
    pub fn process(&self, file: &mut FormatSliceFile, progress: CombinedProgress<2>) {
        self.elephant_foot_fixer
            .post_slice(file, progress[0].clone());
        self.anti_alias.post_slice(file, progress[1].clone());
    }
}
