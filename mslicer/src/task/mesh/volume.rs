use clone_macro::clone;
use common::{progress::Progress, units::CubicMilimeters};
use slicer::mesh::{Mesh, MeshId};

use crate::{
    core::project::model::Model,
    task::{PollResult, Task, TaskApp, TaskStatus, thread::TaskThread},
};

pub struct MeshVolume {
    mesh_id: MeshId,
    progress: Progress,
    handle: TaskThread<CubicMilimeters>,
}

impl MeshVolume {
    pub fn new(model: &Model) -> Self {
        let progress = Progress::new();
        let handle = TaskThread::spawn(clone!([progress, { model.mesh } as mesh], move || {
            mesh_volume(&mesh, &progress)
        }));

        Self {
            mesh_id: model.mesh.mesh_id(),
            progress,
            handle,
        }
    }
}

impl Task for MeshVolume {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        self.handle
            .poll(app, "Failed to find Mesh Volume")
            .into_poll_result(|volume| {
                let models = app.project.models.iter_mut();
                for model in models.filter(|x| x.mesh.mesh_id() == self.mesh_id) {
                    model.set_base_volume(volume);
                }
                PollResult::complete()
            })
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Mesh Volume".into(),
            details: None,
            progress: self.progress.progress(),
        })
    }
}

// Reference: https://stackoverflow.com/a/13927691
fn mesh_volume(mesh: &Mesh, progress: &Progress) -> CubicMilimeters {
    progress.set_total(mesh.face_count() as u64);
    let mut volume = 0.0;

    for face in 0..mesh.face_count() {
        let [a, b, c] = mesh.face_verts(face);
        volume += a.dot(&b.cross(&c)) / 6.0;

        (face & 0x7F == 0).then(|| progress.set_complete(face as u64));
    }

    progress.set_finished();
    CubicMilimeters::new(volume.abs())
}
