use common::progress::Progress;
use egui::{Context, Id, ProgressBar, Ui, Window, vec2};

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

pub fn progress_window(
    ctx: &Context,
    id: Id,
    progress: &Progress,
    title: &str,
    body: impl Fn(&mut Ui),
) {
    let size = vec2(400.0, 0.0);
    Window::new("")
        .id(id)
        .title_bar(false)
        .resizable(false)
        .default_size(size)
        .default_pos((ctx.content_rect().size() - size).to_pos2() / 2.0)
        .show(ctx, |ui| {
            ui.set_height(50.0);
            ui.vertical_centered(|ui| ui.heading(title));
            ui.separator();
            body(ui);
            ui.add(ProgressBar::new(progress.progress()).show_percentage())
        });
}
