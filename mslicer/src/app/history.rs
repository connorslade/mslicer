use std::{borrow::Cow, collections::VecDeque, mem};

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

pub struct ActionDescription {
    pub name: Cow<'static, str>,
    pub extra: Option<Cow<'static, str>>,
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
    pub fn clear(&mut self) {
        self.future.clear();
        self.history.clear();
    }

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

    /// Undoes the action at `index` (as indexed by `History::history`,
    /// oldest first) along with everything performed after it.
    pub fn undo_to(&mut self, index: usize) {
        while self.history.len() > index {
            self.undo();
        }
    }
}

impl ActionDescription {
    pub fn new(name: &'static str) -> Self {
        Self {
            name: Cow::Borrowed(name),
            extra: None,
        }
    }

    pub fn with_extra(self, extra: impl Into<Cow<'static, str>>) -> Self {
        Self {
            extra: Some(extra.into()),
            ..self
        }
    }
}

impl Action {
    pub fn description(&self) -> ActionDescription {
        match self {
            Self::Model { action, .. } => action.description(),
            Self::SliceConfig(action) => action.description(),
            Self::ModelAdded { .. } => ActionDescription::new("Loaded Model"),
            Self::ModelRemoved { model, .. } => {
                ActionDescription::new("Removed Model").with_extra(model.name.clone())
            }
            Self::CollectionAdded { .. } => ActionDescription::new("Created Collection"),
            Self::CollectionRemoved { collection, .. } => {
                ActionDescription::new("Removed Collection").with_extra(collection.name.clone())
            }
        }
    }

    pub fn undo(self, app: &mut App) -> Option<Action> {
        match self {
            Self::Model { id, action } => action
                .undo(app, id)
                .map(|action| Self::Model { id, action }),
            Self::ModelAdded { id } => {
                let index = app.project.models.iter().position(|x| x.id == id)?;
                let model = Box::new(app.project.models.remove(index));
                Some(Self::ModelRemoved { index, model })
            }
            Self::ModelRemoved { index, model } => {
                let id = model.id;
                let index = index.min(app.project.models.len());
                app.project.models.insert(index, *model);
                Some(Self::ModelAdded { id })
            }
            Self::CollectionAdded { id } => {
                let index = app.project.collections.iter().position(|x| x.id == id)?;
                let collection = app.project.collections.remove(index);
                Some(Self::CollectionRemoved { index, collection })
            }
            Self::CollectionRemoved { index, collection } => {
                let id = collection.id;
                let index = index.min(app.project.collections.len());
                app.project.collections.insert(index, collection);
                Some(Self::CollectionAdded { id })
            }
            Self::SliceConfig(action) => action.undo(app).map(Action::SliceConfig),
        }
    }
}

impl SliceConfigAction {
    pub fn description(&self) -> ActionDescription {
        match self {
            Self::Mode(_) => ActionDescription::new("Changed Slice Mode"),
            Self::PlatformResolution(_) => ActionDescription::new("Changed Platform Resolution"),
            Self::PlatformSize(_) => ActionDescription::new("Changed Platform Size"),
            Self::SliceHeight(_) => ActionDescription::new("Changed Slice Height"),
            Self::Supersample(_) => ActionDescription::new("Changed Anti Aliasing"),
            Self::FirstLayers(_) => ActionDescription::new("Changed First Layers"),
            Self::TransitionLayers(_) => ActionDescription::new("Changed Transition Layers"),
            Self::NormalExposure(_) => ActionDescription::new("Changed Normal Layer Exposure"),
            Self::FirstExposure(_) => ActionDescription::new("Changed First Layer Exposure"),
            Self::ExposureRemap(_) => ActionDescription::new("Changed Exposure Remapping"),
            Self::VariableLayerHeight(_) => ActionDescription::new("Changed Variable Layer Height"),
            Self::ElephantFootFixer(_) => ActionDescription::new("Changed Elephant Foot Fixer"),
        }
    }

    pub fn undo(self, app: &mut App) -> Option<SliceConfigAction> {
        let slice_config = &mut app.project.slice_config;
        let post_processing = &mut app.project.post_processing;

        Some(match self {
            Self::Mode(mode) => Self::Mode(mem::replace(&mut slice_config.mode, mode)),
            Self::PlatformResolution(resolution) => Self::PlatformResolution(mem::replace(
                &mut slice_config.platform_resolution,
                resolution,
            )),
            Self::PlatformSize(size) => {
                Self::PlatformSize(mem::replace(&mut slice_config.platform_size, size))
            }
            Self::SliceHeight(height) => {
                Self::SliceHeight(mem::replace(&mut slice_config.slice_height, height))
            }
            Self::Supersample(supersample) => {
                Self::Supersample(mem::replace(&mut slice_config.supersample, supersample))
            }
            Self::FirstLayers(layers) => {
                Self::FirstLayers(mem::replace(&mut slice_config.first_layers, layers))
            }
            Self::TransitionLayers(layers) => {
                Self::TransitionLayers(mem::replace(&mut slice_config.transition_layers, layers))
            }
            Self::NormalExposure(config) => {
                Self::NormalExposure(mem::replace(&mut slice_config.exposure_config, config))
            }
            Self::FirstExposure(config) => Self::FirstExposure(mem::replace(
                &mut slice_config.first_exposure_config,
                config,
            )),
            Self::ExposureRemap(remap) => {
                Self::ExposureRemap(mem::replace(&mut slice_config.exposure_remap, remap))
            }
            Self::VariableLayerHeight(value) => Self::VariableLayerHeight(mem::replace(
                &mut post_processing.variable_layer_height,
                value,
            )),
            Self::ElephantFootFixer(value) => Self::ElephantFootFixer(mem::replace(
                &mut post_processing.elephant_foot_fixer,
                value,
            )),
        })
    }
}

impl ModelAction {
    pub fn description(&self) -> ActionDescription {
        match self {
            Self::Name(name) => ActionDescription::new("Renamed Model").with_extra(name.clone()),
            Self::Color(_) => ActionDescription::new("Changed Model Color"),
            Self::Hidden(hide) => {
                ActionDescription::new(if *hide { "Showed Model" } else { "Hid Model" })
            }
            Self::Position(_) => ActionDescription::new("Moved Model"),
            Self::Scale(_) => ActionDescription::new("Scaled Model"),
            Self::Rotation(_) => ActionDescription::new("Rotated Model"),
            Self::RelativeExposure(_) => ActionDescription::new("Changed Model Exposure"),
            Self::Collection(_) => ActionDescription::new("Changed Model Collection"),
            Self::Move(..) => ActionDescription::new("Moved Model"),
        }
    }

    /// Undoes the model action on the specified model, returning an action to
    /// revert the undo (redo) if the model was found.
    pub fn undo(self, app: &mut App, model_id: ModelId) -> Option<ModelAction> {
        let model = app.project.models.iter_mut().find(|x| x.id == model_id)?;
        let platform_size = &app.project.slice_config.platform_size;

        Some(match self {
            Self::Name(name) => Self::Name(mem::replace(&mut model.name, name)),
            Self::Color(color) => Self::Color(mem::replace(&mut model.color, color)),
            Self::Hidden(hide) => Self::Hidden(mem::replace(&mut model.hidden, hide)),
            Self::Position(matrix) => {
                let old = model.mesh.position();
                model.set_position(platform_size, matrix);
                Self::Position(old)
            }
            Self::Scale(matrix) => {
                let old = model.mesh.scale();
                model.set_scale(platform_size, matrix);
                Self::Scale(old)
            }
            Self::Rotation(matrix) => {
                let old = model.mesh.rotation();
                model.set_rotation(platform_size, matrix);
                Self::Rotation(old)
            }
            Self::RelativeExposure(exposure) => {
                Self::RelativeExposure(mem::replace(&mut model.exposure, exposure))
            }
            Self::Collection(collection) => {
                Self::Collection(mem::replace(&mut model.collection, collection))
            }
            Self::Move(index, collection) => {
                let current_index = app.project.models.iter().position(|x| x.id == model_id)?;
                let mut model = app.project.models.remove(current_index);
                let old_collection = mem::replace(&mut model.collection, collection);
                let index = index.min(app.project.models.len());
                app.project.models.insert(index, model);
                return Some(Self::Move(current_index, old_collection));
            }
        })
    }
}

impl From<SliceConfigAction> for Action {
    fn from(value: SliceConfigAction) -> Self {
        Self::SliceConfig(value)
    }
}
