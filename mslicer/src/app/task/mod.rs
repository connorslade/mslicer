use egui::Context;

use crate::app::App;

mod file_dialog;
mod mesh_load;
mod mesh_manifold;
pub use self::{file_dialog::FileDialog, mesh_load::MeshLoad, mesh_manifold::MeshManifold};

// Async operation that can be polled every frame.
pub trait Task {
    /// Returns true if the task has completed.
    fn poll(&mut self, app: &mut App, ctx: &Context) -> bool;
}

#[derive(Default)]
pub struct TaskManager {
    tasks: Vec<Box<dyn Task>>,
}

impl TaskManager {
    pub fn add(&mut self, task: impl Task + 'static) {
        self.tasks.push(Box::new(task));
    }

    pub(super) fn render(&mut self, app: &mut App, ctx: &Context) {
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
