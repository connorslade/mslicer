use std::{
    any::Any,
    thread::{self, JoinHandle},
};

use crate::{
    interface::popup::{Popup, PopupIcon},
    task::{PollResult, TaskApp},
};

pub struct TaskThread<T> {
    handle: Handle<T>,
}

enum Handle<T> {
    Thread(Option<JoinHandle<T>>),
    Dummy(Option<T>),
}

pub enum TaskResult<T> {
    Completed(T),
    Empty,

    Failed,
    Pending,
}

impl<T: Send + 'static> TaskThread<T> {
    pub fn spawn(f: impl FnOnce() -> T + Send + 'static) -> Self {
        let handle = thread::Builder::new()
            .name("task_thread".into())
            .spawn(f)
            .unwrap();
        Self {
            handle: Handle::Thread(Some(handle)),
        }
    }

    pub fn complete(value: T) -> Self {
        Self {
            handle: Handle::Dummy(Some(value)),
        }
    }

    pub fn poll(&mut self, app: &mut TaskApp, failure: &str) -> TaskResult<T> {
        if self.handle.is_finished() {
            match self.handle.join() {
                Some(Ok(value)) => TaskResult::Completed(value),
                Some(Err(err)) => {
                    let body = if let Some(err) = err.downcast_ref::<String>() {
                        err.clone()
                    } else if let Some(err) = err.downcast_ref::<&str>() {
                        err.to_string()
                    } else {
                        format!("{err:?}")
                    };

                    app.popup
                        .open(Popup::simple(failure, PopupIcon::Error, body));
                    TaskResult::Failed
                }
                None => TaskResult::Empty,
            }
        } else {
            TaskResult::Pending
        }
    }

    /// Polls the task thread without any special handing of errors, they are
    /// just ignored.
    pub fn poll_ignore_err(&mut self) -> TaskResult<T> {
        if self.handle.is_finished() {
            match self.handle.join() {
                Some(Ok(value)) => TaskResult::Completed(value),
                Some(Err(_)) => TaskResult::Failed,
                None => TaskResult::Empty,
            }
        } else {
            TaskResult::Pending
        }
    }
}

impl<T> Handle<T> {
    pub fn is_finished(&self) -> bool {
        match self {
            Handle::Thread(Some(handle)) => handle.is_finished(),
            _ => true,
        }
    }

    pub fn join(&mut self) -> Option<Result<T, Box<dyn Any + Send + 'static>>> {
        match self {
            Handle::Thread(handle) => handle.take().map(|h| h.join()),
            Handle::Dummy(value) => value.take().map(Ok),
        }
    }
}

impl<T> TaskResult<T> {
    pub fn into_poll_result(self, callback: impl FnOnce(T) -> PollResult) -> PollResult {
        match self {
            TaskResult::Completed(value) => callback(value),
            TaskResult::Failed | TaskResult::Empty => PollResult::complete(),
            TaskResult::Pending => PollResult::pending(),
        }
    }
}
