use std::borrow::Cow;

use crate::{
    app::{App, config::Config},
    app_ref_type,
    project::Project,
    ui::{popup::PopupManager, state::UiState},
};

mod acceleration_structures;
mod file_dialog;
mod island_detection;
mod mesh_load;
mod mesh_manifold;
mod project;
mod remote_print;
mod save_result;
mod thread;
mod webhook;
pub use self::{
    acceleration_structures::BuildAccelerationStructures,
    file_dialog::FileDialog,
    island_detection::IslandDetection,
    mesh_load::MeshLoad,
    mesh_manifold::MeshManifold,
    project::{ProjectLoad, ProjectSave},
    remote_print::{PrinterConnect, PrinterScan},
    save_result::SaveResult,
    webhook::Webhook,
};

// Async operation that can be polled every frame.
pub trait Task {
    /// Returns true if the task has completed.
    fn poll(&mut self, app: &mut TaskApp) -> PollResult;

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

/// A subset of App fields, excluding `tasks`. This allows mutable access to
/// these fields in task callbacks without two mutable references to the
/// TaskManager.
pub struct TaskApp<'a> {
    pub popup: &'a mut PopupManager,
    pub state: &'a mut UiState,
    pub config: &'a mut Config,
    pub project: &'a mut Project,
}

app_ref_type!(TaskManager, tasks);

impl TaskManager {
    pub fn add(&mut self, task: impl Task + 'static) {
        self.add_boxed(Box::new(task));
    }

    pub fn add_boxed(&mut self, task: Box<dyn Task>) {
        self.tasks.push(task);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Box<dyn Task>> {
        self.tasks.iter()
    }

    pub fn any_with_status(&self) -> bool {
        self.iter().any(|x| x.status().is_some())
    }

    pub fn progress(&self) -> f32 {
        let (mut t, mut n) = (0.0, 0);
        for task in self.tasks.iter() {
            if let Some(status) = task.status() {
                t += status.progress;
                n += 1;
            }
        }

        if n == 0 { 0.0 } else { t / n as f32 }
    }
}

impl<'a> TaskManagerRef<'a> {
    pub(super) fn poll(&mut self) {
        let this = &mut self.app.tasks;
        let mut app = TaskApp {
            popup: &mut self.app.popup,
            state: &mut self.app.state,
            config: &mut self.app.config,
            project: &mut self.app.project,
        };

        let mut i = 0;
        while i < this.tasks.len() {
            let task = &mut this.tasks[i];
            let result = task.poll(&mut app);
            if result.complete {
                this.tasks.remove(i);
            } else {
                i += 1;
            }

            this.tasks.extend(result.new_tasks);
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

    pub fn with_tasks(mut self, task: Vec<Box<dyn Task>>) -> Self {
        self.new_tasks.extend(task);
        self
    }
}
