use std::borrow::Cow;

use crate::app::App;

mod acceleration_structures;
mod file_dialog;
mod mesh_load;
mod mesh_manifold;
mod project;
pub use self::{
    acceleration_structures::BuildAccelerationStructures,
    file_dialog::FileDialog,
    mesh_load::MeshLoad,
    mesh_manifold::MeshManifold,
    project::{ProjectLoad, ProjectSave},
};

// Async operation that can be polled every frame.
pub trait Task {
    /// Returns true if the task has completed.
    fn poll(&mut self, app: &mut App) -> PollResult;

    fn status(&self) -> Option<TaskStatus<'_>> {
        None
    }
}

pub struct PollResult {
    complete: bool,
    new_tasks: Vec<Box<dyn Task>>,
}

pub struct TaskStatus<'a> {
    pub name: Cow<'a, str>,
    pub details: Option<String>,
    pub progress: f32,
}

#[derive(Default)]
pub struct TaskManager {
    tasks: Vec<Box<dyn Task>>,
}

impl TaskManager {
    pub fn add(&mut self, task: impl Task + 'static) {
        self.tasks.push(Box::new(task));
    }

    pub fn iter(&self) -> impl Iterator<Item = &Box<dyn Task>> {
        self.tasks.iter()
    }

    pub fn any_with_status(&self) -> bool {
        self.iter().any(|x| x.status().is_some())
    }

    pub(super) fn poll(&mut self, app: &mut App) {
        let mut i = 0;
        while i < self.tasks.len() {
            let result = self.tasks[i].poll(app);
            if result.complete {
                self.tasks.remove(i);
            } else {
                i += 1;
            }

            self.tasks.extend(result.new_tasks);
        }
    }
}

impl PollResult {
    pub fn from_bool(complete: bool) -> Self {
        Self {
            complete,
            new_tasks: Vec::new(),
        }
    }

    pub fn pending() -> Self {
        Self::from_bool(false)
    }

    pub fn complete() -> Self {
        Self::from_bool(true)
    }

    pub fn with_task(mut self, task: impl Task + 'static) -> Self {
        self.new_tasks.push(Box::new(task));
        self
    }
}
