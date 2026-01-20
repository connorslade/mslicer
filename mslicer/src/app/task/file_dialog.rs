use std::path::Path;

use poll_promise::Promise;
use rfd::{AsyncFileDialog, FileHandle};

use crate::app::{
    App,
    task::{PollResult, Task},
};

type Callback = Box<dyn FnOnce(&mut App, &Path)>;

pub struct FileDialog {
    func: Option<Callback>,
    promise: Promise<Option<FileHandle>>,
}

impl FileDialog {
    fn new(
        file: impl Future<Output = Option<FileHandle>> + Send + 'static,
        callback: impl FnMut(&mut App, &Path) + 'static,
    ) -> Self {
        let promise = Promise::spawn_async(file);
        FileDialog {
            func: Some(Box::new(callback)),
            promise,
        }
    }

    pub fn pick_file(
        (name, extensions): (impl Into<String>, &[impl ToString]),
        callback: impl FnMut(&mut App, &Path) + 'static,
    ) -> Self {
        let file = AsyncFileDialog::new()
            .add_filter(name, extensions)
            .pick_file();
        Self::new(file, callback)
    }

    pub fn save_file(
        (name, extensions): (impl Into<String>, &[impl ToString]),
        callback: impl FnMut(&mut App, &Path) + 'static,
    ) -> Self {
        let file = AsyncFileDialog::new()
            .add_filter(name, extensions)
            .save_file();
        Self::new(file, callback)
    }
}

impl Task for FileDialog {
    fn poll(&mut self, app: &mut App) -> PollResult {
        let result = self.promise.ready();
        if let Some(data) = result
            && let Some(handle) = data
        {
            let path = handle.path();
            self.func.take().unwrap()(app, path);
        }

        PollResult::from_bool(result.is_some())
    }
}
