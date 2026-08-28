use std::{collections::VecDeque, mem};

use common::{
    color::LinearRgb,
    slice::{ExposureConfig, ExposureRemap, SliceMode},
    units::Milimeters,
};
use nalgebra::{Vector2, Vector3};
use slicer::post_process::{
    elephant_foot_fixer::ElephantFootFixer, variable_layer_height::VariableLayerHeight,
};

use crate::{
    app::App,
    app_ref_type,
    project::{
        Collection, CollectionId,
        model::{Model, ModelId},
    },
};

const MAX_HISTORY: usize = 0x80; // random number i picked

#[derive(Default)]
pub struct History {
    pub history: VecDeque<Action>,
    future: VecDeque<Action>,
}

app_ref_type!(History, history);

#[derive(Clone, PartialEq)]
pub enum Action {
    Model {
        id: ModelId,
        action: ModelAction,
    },
    SliceConfig(SliceConfigAction),

    ModelAdded {
        id: ModelId,
    },
    ModelRemoved {
        index: usize,
        model: Box<Model>,
    },
    CollectionAdded {
        id: CollectionId,
    },
    CollectionRemoved {
        index: usize,
        collection: Collection,
    },
}

#[derive(Clone, PartialEq)]
pub enum SliceConfigAction {
    Mode(SliceMode),
    PlatformResolution(Vector2<u32>),
    PlatformSize(Vector3<Milimeters>),
    SliceHeight(Milimeters),
    Supersample(u8),
    FirstLayers(u32),
    TransitionLayers(u32),
    NormalExposure(ExposureConfig),
    FirstExposure(ExposureConfig),
    ExposureRemap(ExposureRemap),
    VariableLayerHeight(VariableLayerHeight),
    ElephantFootFixer(ElephantFootFixer),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModelAction {
    Name(String),
    Color(LinearRgb<f32>),
    Hidden(bool),
    Position(Vector3<f32>),
    Scale(Vector3<f32>),
    Rotation(Vector3<f32>),
    RelativeExposure(u8),
    Collection(Option<CollectionId>),
    Move(usize, Option<CollectionId>),
}

impl History {
    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    /// Ensure the list of past and future actions is not greater than
    /// MAX_HISTORY.
    fn constrain_size(&mut self) {
        while self.history.len() >= MAX_HISTORY {
            self.history.pop_front();
        }

        while self.future.len() >= MAX_HISTORY {
            self.future.pop_front();
        }
    }

    pub fn track(&mut self, action: Action) {
        self.constrain_size();
        self.history.push_back(action);
        self.future.clear();
    }

    pub fn track_model(&mut self, id: ModelId, action: ModelAction) {
        self.track(Action::Model { id, action });
    }
}

impl<'a> HistoryRef<'a> {
    pub fn undo(&mut self) {
        if let Some(redo) = (self.history.pop_back()).and_then(|action| action.undo(self.app)) {
            self.constrain_size();
            self.future.push_back(redo);
        }
    }

    pub fn redo(&mut self) {
        if let Some(redo) = (self.future.pop_back()).and_then(|action| action.undo(self.app)) {
            self.constrain_size();
            self.history.push_back(redo);
        }
    }
}

impl Action {
    pub fn undo(self, app: &mut App) -> Option<Action> {
        match self {
            Action::Model { id, action } => action
                .undo(app, id)
                .map(|action| Action::Model { id, action }),
            Action::ModelAdded { id } => {
                let index = app.project.models.iter().position(|x| x.id == id)?;
                let model = Box::new(app.project.models.remove(index));
                Some(Action::ModelRemoved { index, model })
            }
            Action::ModelRemoved { index, model } => {
                let id = model.id;
                let index = index.min(app.project.models.len());
                app.project.models.insert(index, *model);
                Some(Action::ModelAdded { id })
            }
            Action::CollectionAdded { id } => {
                let index = app.project.collections.iter().position(|x| x.id == id)?;
                let collection = app.project.collections.remove(index);
                Some(Action::CollectionRemoved { index, collection })
            }
            Action::CollectionRemoved { index, collection } => {
                let id = collection.id;
                let index = index.min(app.project.collections.len());
                app.project.collections.insert(index, collection);
                Some(Action::CollectionAdded { id })
            }
            Action::SliceConfig(action) => action.undo(app).map(Action::SliceConfig),
        }
    }
}

impl SliceConfigAction {
    pub fn undo(self, app: &mut App) -> Option<SliceConfigAction> {
        let slice_config = &mut app.project.slice_config;
        let post_processing = &mut app.project.post_processing;

        Some(match self {
            SliceConfigAction::Mode(mode) => {
                SliceConfigAction::Mode(mem::replace(&mut slice_config.mode, mode))
            }
            SliceConfigAction::PlatformResolution(resolution) => {
                SliceConfigAction::PlatformResolution(mem::replace(
                    &mut slice_config.platform_resolution,
                    resolution,
                ))
            }
            SliceConfigAction::PlatformSize(size) => {
                SliceConfigAction::PlatformSize(mem::replace(&mut slice_config.platform_size, size))
            }
            SliceConfigAction::SliceHeight(height) => {
                SliceConfigAction::SliceHeight(mem::replace(&mut slice_config.slice_height, height))
            }
            SliceConfigAction::Supersample(supersample) => SliceConfigAction::Supersample(
                mem::replace(&mut slice_config.supersample, supersample),
            ),
            SliceConfigAction::FirstLayers(layers) => {
                SliceConfigAction::FirstLayers(mem::replace(&mut slice_config.first_layers, layers))
            }
            SliceConfigAction::TransitionLayers(layers) => SliceConfigAction::TransitionLayers(
                mem::replace(&mut slice_config.transition_layers, layers),
            ),
            SliceConfigAction::NormalExposure(config) => SliceConfigAction::NormalExposure(
                mem::replace(&mut slice_config.exposure_config, config),
            ),
            SliceConfigAction::FirstExposure(config) => SliceConfigAction::FirstExposure(
                mem::replace(&mut slice_config.first_exposure_config, config),
            ),
            SliceConfigAction::ExposureRemap(remap) => SliceConfigAction::ExposureRemap(
                mem::replace(&mut slice_config.exposure_remap, remap),
            ),
            SliceConfigAction::VariableLayerHeight(value) => {
                SliceConfigAction::VariableLayerHeight(mem::replace(
                    &mut post_processing.variable_layer_height,
                    value,
                ))
            }
            SliceConfigAction::ElephantFootFixer(value) => SliceConfigAction::ElephantFootFixer(
                mem::replace(&mut post_processing.elephant_foot_fixer, value),
            ),
        })
    }
}

impl ModelAction {
    /// Undoes the model action on the specified model, returning an action to
    /// revert the undo (redo) if the model was found.
    pub fn undo(self, app: &mut App, model_id: ModelId) -> Option<ModelAction> {
        let model = app.project.models.iter_mut().find(|x| x.id == model_id)?;
        let platform_size = &app.project.slice_config.platform_size;

        Some(match self {
            ModelAction::Name(name) => ModelAction::Name(mem::replace(&mut model.name, name)),
            ModelAction::Color(color) => ModelAction::Color(mem::replace(&mut model.color, color)),
            ModelAction::Hidden(hide) => ModelAction::Hidden(mem::replace(&mut model.hidden, hide)),
            ModelAction::Position(matrix) => {
                let old = model.mesh.position();
                model.set_position(platform_size, matrix);
                ModelAction::Position(old)
            }
            ModelAction::Scale(matrix) => {
                let old = model.mesh.scale();
                model.set_scale(platform_size, matrix);
                ModelAction::Scale(old)
            }
            ModelAction::Rotation(matrix) => {
                let old = model.mesh.rotation();
                model.set_rotation(platform_size, matrix);
                ModelAction::Rotation(old)
            }
            ModelAction::RelativeExposure(exposure) => {
                ModelAction::RelativeExposure(mem::replace(&mut model.exposure, exposure))
            }
            ModelAction::Collection(collection) => {
                ModelAction::Collection(mem::replace(&mut model.collection, collection))
            }
            ModelAction::Move(index, collection) => {
                let current_index = app.project.models.iter().position(|x| x.id == model_id)?;
                let mut model = app.project.models.remove(current_index);
                let old_collection = mem::replace(&mut model.collection, collection);
                let index = index.min(app.project.models.len());
                app.project.models.insert(index, model);
                return Some(ModelAction::Move(current_index, old_collection));
            }
        })
    }
}

impl From<SliceConfigAction> for Action {
    fn from(value: SliceConfigAction) -> Self {
        Self::SliceConfig(value)
    }
}
