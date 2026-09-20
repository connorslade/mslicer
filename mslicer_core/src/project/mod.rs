use std::path::PathBuf;

use crate::project::model::{Model, ModelId};
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
pub mod supports;

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

impl PartialEq for Collection {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
