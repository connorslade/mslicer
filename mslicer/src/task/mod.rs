use std::{
    borrow::Cow,
    sync::mpsc::{self, Receiver, SyncSender},
};

use crate::{
    app::{config::Config, history::History, slice_operation::SliceOperation},
    app_ref_type,
    project::Project,
    ui::{panels::Panels, popup::PopupManager, state::UiState},
};

mod file;
mod mesh;
mod network;
mod slice;
mod thread;
pub use self::{file::*, mesh::*, network::*, slice::*};

type TaskQueue = (
    SyncSender<Box<dyn Task + Send + Sync>>,
    Receiver<Box<dyn Task + Send + Sync>>,
);

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

pub struct TaskManager {
    /// List of current tasks that get polled every frame.
    tasks: Vec<Box<dyn Task>>,
    /// MPSC channel to add tasks to be polled from the UI from async threads.
    task_queue: TaskQueue,
}

/// A subset of App fields, excluding `tasks`. This allows mutable access to
/// these fields in task callbacks without two mutable references to the
/// TaskManager.
pub struct TaskApp<'a> {
    pub panels: &'a mut Panels,
    pub popup: &'a mut PopupManager,
    pub slice_operation: &'a mut Option<SliceOperation>,
    pub state: &'a mut UiState,
    pub config: &'a mut Config,
    pub project: &'a mut Project,
    pub history: &'a mut History,
}

app_ref_type!(TaskManager, tasks);

impl TaskManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::sync_channel(16);
        Self {
            tasks: Vec::new(),
            task_queue: (tx, rx),
        }
    }

    pub fn sender(&self) -> SyncSender<Box<dyn Task + Send + Sync>> {
        self.task_queue.0.clone()
    }

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
        while let Ok(pending) = this.task_queue.1.try_recv() {
            this.add_boxed(pending);
        }

        let mut app = TaskApp {
            panels: &mut self.app.panels,
            popup: &mut self.app.popup,
            slice_operation: &mut self.app.slice_operation,
            state: &mut self.app.state,
            config: &mut self.app.config,
            project: &mut self.app.project,
            history: &mut self.app.history,
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
