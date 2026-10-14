use clone_macro::clone;
use common::{
    progress::{CombinedProgress, Progress},
    units::CubicMilimeters,
};
use slicer::geometry::mesh::{Mesh, MeshId};

use crate::{
    core::project::model::{MeshWarnings, Model},
    task::{PollResult, Task, TaskApp, TaskStatus, thread::TaskThread},
};

pub struct MeshAnalysis {
    mesh_id: MeshId,
    progress: CombinedProgress<2>,

    defective: TaskThread<bool>,
    volume: TaskThread<CubicMilimeters>,
}

impl MeshAnalysis {
    pub fn new(mesh: &Model) -> Self {
        let progress = CombinedProgress::new();

        let defective = TaskThread::spawn(clone!(
            [{ progress[0] } as progress, { mesh.mesh } as mesh],
            move || { mesh.is_defective(progress) }
        ));

        let volume = TaskThread::spawn(clone!(
            [{ progress[1] } as progress, { mesh.mesh } as mesh],
            move || { mesh_volume(&mesh, &progress) }
        ));

        Self {
            mesh_id: mesh.mesh.mesh_id(),
            progress,

            defective,
            volume,
        }
    }
}

impl Task for MeshAnalysis {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        let defective_result = self
            .defective
            .poll(app, "Failed to Check for Mesh Defects")
            .into_poll_result(|defective| {
                let models = app.project.models.iter_mut();
                for model in models.filter(|x| x.mesh.mesh_id() == self.mesh_id) {
                    model.warnings.set(MeshWarnings::Defective, defective);
                }
                PollResult::complete()
            });

        let volume_result = self
            .volume
            .poll(app, "Failed to find Mesh Volume")
            .into_poll_result(|volume| {
                let models = app.project.models.iter_mut();
                for model in models.filter(|x| x.mesh.mesh_id() == self.mesh_id) {
                    model.set_base_volume(volume);
                }
                PollResult::complete()
            });

        defective_result.and(volume_result)
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Mesh Analysis".into(),
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
