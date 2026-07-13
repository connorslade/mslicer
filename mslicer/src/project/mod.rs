use std::path::PathBuf;

use crate::{
    project::model::{Model, ModelId},
    task::{FileDialog, ProjectLoad, ProjectSave, Task},
};
use common::{
    id_type,
    progress::CombinedProgress,
    slice::{Layer, SliceConfig},
};
use slicer::post_process::{
    elephant_foot_fixer::ElephantFootFixer, variable_layer_height::VariableLayerHeight,
};

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
    pub id: CollectionId,
    pub name: String,
    pub collapsed: bool,

    pub rename: RenameState, // not persistent
}

#[derive(Default, Clone)]
pub struct PostProcessing {
    pub variable_layer_height: VariableLayerHeight,
    pub elephant_foot_fixer: ElephantFootFixer,
}

#[derive(Default, Clone)]
pub enum RenameState {
    #[default]
    None,
    Starting,
    Editing,
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
        self.collections.clear();
    }

    pub fn model(&mut self, id: ModelId) -> Option<&mut Model> {
        self.models.iter_mut().find(|x| x.id == id)
    }

    pub fn collection(&mut self, id: CollectionId) -> Option<&mut Collection> {
        self.collections.iter_mut().find(|x| x.id == id)
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
            id: CollectionId::new(),
            name,
            collapsed: false,
            rename: RenameState::None,
        }
    }

    pub fn new_unnamed() -> Self {
        Self::new("Collection".into())
    }
}

impl PostProcessing {
    pub fn process(
        &self,
        config: &SliceConfig,
        layers: &mut Vec<Layer>,
        progress: CombinedProgress<2>,
    ) {
        self.variable_layer_height
            .post_slice(config, layers, progress[0].clone());
        self.elephant_foot_fixer
            .post_slice(config, layers, progress[1].clone());
    }
}

id_type!(CollectionId, u32);
