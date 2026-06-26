use std::{
    path::PathBuf,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::{
    project::model::Model,
    task::{FileDialog, ProjectLoad, ProjectSave, Task},
};
use common::{
    progress::CombinedProgress,
    slice::{Layer, SliceConfig},
};
use slicer::post_process::elephant_foot_fixer::ElephantFootFixer;

pub mod model;
pub mod storage;

#[derive(Default, Clone)]
pub struct Project {
    pub path: Option<PathBuf>,
    pub slice_config: SliceConfig,
    pub post_processing: PostProcessing,
    pub models: Vec<Model>,
    pub collections: Vec<Collection>,
}

#[derive(Default, Clone)]
pub struct Collection {
    pub id: u32,
    pub name: String,
    pub collapsed: bool,
}

#[derive(Default, Clone)]
pub struct PostProcessing {
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

impl Collection {
    pub fn new(name: String) -> Self {
        Self {
            id: next_id(),
            name,
            collapsed: false,
        }
    }
}

impl PostProcessing {
    pub fn process(
        &self,
        config: &SliceConfig,
        layers: &mut [Layer],
        progress: CombinedProgress<1>,
    ) {
        self.elephant_foot_fixer
            .post_slice(config, layers, progress[0].clone());
    }
}

fn next_id() -> u32 {
    static NEXT_ID: AtomicU32 = AtomicU32::new(0);
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}
