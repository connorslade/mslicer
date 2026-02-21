use std::{collections::VecDeque, mem};

use common::color::LinearRgb;
use nalgebra::Vector3;

use crate::{app::App, app_ref_type};

const MAX_HISTORY: usize = 0x80; // random number i picked

#[derive(Default)]
pub struct History {
    pub history: VecDeque<Action>,
    future: VecDeque<Action>,
}

app_ref_type!(History, history);

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Model { id: u32, action: ModelAction },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModelAction {
    Name(String),
    Color(LinearRgb<f32>),
    Hidden(bool),
    Position(Vector3<f32>),
    Scale(Vector3<f32>),
    Rotation(Vector3<f32>),
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

    pub fn track_model(&mut self, id: u32, action: ModelAction) {
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
        }
    }
}

impl ModelAction {
    /// Undoes the model action on the specified model, returning an action to
    /// revert the undo (redo) if the model was found.
    pub fn undo(self, app: &mut App, model: u32) -> Option<ModelAction> {
        let model = app.project.models.iter_mut().find(|x| x.id == model)?;
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
        })
    }
}
