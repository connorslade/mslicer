use std::thread::{self, JoinHandle};

use crate::{
    app::{App, task::PollResult},
    ui::popup::{Popup, PopupIcon},
};

pub struct TaskThread<T> {
    handle: Option<JoinHandle<T>>,
}

pub enum TaskResult<T> {
    Completed(T),
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
            handle: Some(handle),
        }
    }

    pub fn poll(&mut self, app: &mut App, failure: &str) -> TaskResult<T> {
        let handle = self.handle.as_ref().unwrap();
        if handle.is_finished() {
            let handle = self.handle.take().unwrap();
            match handle.join() {
                Ok(value) => TaskResult::Completed(value),
                Err(err) => {
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
            }
        } else {
            TaskResult::Pending
        }
    }
}

impl<T> TaskResult<T> {
    pub fn into_poll_result(self, callback: impl FnOnce(T) -> PollResult) -> PollResult {
        match self {
            TaskResult::Completed(value) => callback(value),
            TaskResult::Failed => PollResult::complete(),
            TaskResult::Pending => PollResult::pending(),
        }
    }
}
