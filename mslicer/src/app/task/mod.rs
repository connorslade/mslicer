use std::borrow::Cow;

use egui::Context;

use crate::app::App;

mod file_dialog;
mod mesh_load;
mod mesh_manifold;
mod project;
pub use self::{
    file_dialog::FileDialog,
    mesh_load::MeshLoad,
    mesh_manifold::MeshManifold,
    project::{ProjectLoad, ProjectSave},
};

// Async operation that can be polled every frame.
pub trait Task {
    /// Returns true if the task has completed.
    fn poll(&mut self, app: &mut App, ctx: &Context) -> bool;

    fn status(&self) -> Option<TaskStatus<'_>> {
        None
    }
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

    pub(super) fn poll(&mut self, app: &mut App, ctx: &Context) {
        let mut i = 0;
        while i < self.tasks.len() {
            let mut task = self.tasks.remove(i);
            if !task.poll(app, ctx) {
                self.tasks.push(task);
                i += 1;
            }
        }
    }
}
