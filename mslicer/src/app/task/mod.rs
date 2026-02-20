use std::{borrow::Cow, mem};

use nalgebra::Vector2;

use crate::{
    app::{App, config::Config, project::Project},
    app_ref_type,
    ui::popup::PopupManager,
    windows::Tab,
};

mod acceleration_structures;
mod file_dialog;
mod island_detection;
mod mesh_load;
mod mesh_manifold;
mod project;
mod save_result;
mod thread;
pub use self::{
    acceleration_structures::BuildAccelerationStructures,
    file_dialog::FileDialog,
    island_detection::IslandDetection,
    mesh_load::MeshLoad,
    mesh_manifold::MeshManifold,
    project::{ProjectLoad, ProjectSave},
    save_result::SaveResult,
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
    visible_count: usize,
}

/// A subset of App fields, excluding `tasks`. This allows mutable access to
/// these fields in task callbacks without two mutable references to the
/// TaskManager.
pub struct TaskApp<'a> {
    pub popup: &'a mut PopupManager,
    pub config: &'a mut Config,
    pub project: &'a mut Project,
}

app_ref_type!(TaskManager, tasks);

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
}

impl<'a> TaskManagerRef<'a> {
    pub(super) fn poll(&mut self) {
        let this = &mut self.app.tasks;
        let mut app = TaskApp {
            popup: &mut self.app.popup,
            config: &mut self.app.config,
            project: &mut self.app.project,
        };

        let mut i = 0;
        let mut visible = 0;
        while i < this.tasks.len() {
            let task = &mut this.tasks[i];
            visible += task.status().is_some() as usize;
            let result = task.poll(&mut app);
            if result.complete {
                this.tasks.remove(i);
            } else {
                i += 1;
            }

            this.tasks.extend(result.new_tasks);
        }

        if visible > mem::replace(&mut this.visible_count, visible)
            && self.app.dock_state.find_tab(&Tab::Tasks).is_none()
        {
            self.app.add_tab(Tab::Tasks, Vector2::new(400.0, 200.0));
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
