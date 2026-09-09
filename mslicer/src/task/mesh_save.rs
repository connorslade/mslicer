use std::{fs::File, io::BufWriter, mem, path::PathBuf, sync::Arc};

use clone_macro::clone;
use common::{progress::Progress, serde::WriterSerializer};
use mesh_format::{Format, save_mesh};
use slicer::mesh::MeshInner;

use crate::task::{PollResult, Task, TaskApp, TaskStatus, thread::TaskThread};

pub struct MeshSave {
    progress: Progress,
    handle: TaskThread<()>,
}

impl MeshSave {
    pub fn new(path: PathBuf, mesh: Arc<MeshInner>) -> Self {
        let format = path.extension().unwrap().to_string_lossy();
        let format = Format::from_extension(&format).unwrap();

        // SAFETY: Both MeshInner and mesh_format::Mesh have the same layout.
        let mesh = unsafe { mem::transmute::<Arc<MeshInner>, Arc<mesh_format::Mesh>>(mesh) };
        let progress = Progress::new();

        let handle = TaskThread::spawn(clone!([progress], move || {
            let file = File::create(path).unwrap();
            let mut ser = WriterSerializer::new(BufWriter::new(file));
            save_mesh(&mut ser, format, &progress, &mesh);
        }));

        Self { progress, handle }
    }
}

impl Task for MeshSave {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        self.handle
            .poll(app, "Failed to Save Mesh")
            .into_poll_result(|_| PollResult::complete())
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Saving Mesh".into(),
            details: None, // todo: use file name?
            progress: self.progress.progress(),
        })
    }
}
