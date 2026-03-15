use std::path::PathBuf;

use crate::app::{
    project::model::Model,
    task::{FileDialog, ProjectLoad, ProjectSave, Task},
};
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

    pub fn reset(&mut self, default_config: &SliceConfig) {
        self.path = None;
        self.slice_config = default_config.clone();
        self.post_processing = Default::default();
        self.models.clear();
    }
}

impl Project {
    pub fn load() -> FileDialog {
        FileDialog::pick_file(("mslicer project", &["mslicer"]), |_app, path, tasks| {
            tasks.push(Box::new(ProjectLoad::new(path.to_path_buf())))
        })
    }

    pub fn save(&self) -> Box<dyn Task> {
        if let Some(path) = self.path.clone() {
            Box::new(ProjectSave::new(self.clone(), path.to_path_buf()))
        } else {
            Box::new(self.save_as())
        }
    }

    pub fn save_as(&self) -> FileDialog {
        FileDialog::save_file(("mslicer project", &["mslicer"]), |app, path, tasks| {
            let path = path.with_extension("mslicer");
            tasks.push(Box::new(ProjectSave::new(
                app.project.clone(),
                path.to_path_buf(),
            )));
            app.project.path = Some(path);
        })
    }
}

impl PostProcessing {
    pub fn process(&self, file: &mut DynSlicedFile, progress: CombinedProgress<2>) {
        self.elephant_foot_fixer
            .post_slice(file, progress[0].clone());
        self.anti_alias.post_slice(file, progress[1].clone());
    }
}
