use clone_macro::clone;
use common::progress::Progress;
use tools::repair::{self, RepairResult};
use tracing::info;

use crate::{
    core::project::model::{Model, ModelId},
    interface::{components::grid, popup::Popup},
    task::{
        BuildAccelerationStructures, MeshDefective, PollResult, Task, TaskApp, TaskStatus,
        thread::TaskThread,
    },
};

pub struct MeshRepair {
    handle: TaskThread<RepairResult>,
    progress: Progress,

    model: ModelId,
}

impl MeshRepair {
    pub fn new(model: &Model) -> Self {
        let mesh = model.mesh.clone();
        let model = model.id;

        let progress = Progress::new();
        let handle = TaskThread::spawn(clone!([progress], move || {
            progress.set_total(1);
            let result = repair::MeshRepair {
                vertex_epsilon: 1e-4,
            }
            .repair(&mesh);
            progress.set_finished();

            info!(
                "{{ unwelded_vertices: {}, holes: {} }}",
                result.unwelded_vertices, result.holes
            );
            result
        }));

        Self {
            handle,
            progress,
            model,
        }
    }
}

impl Task for MeshRepair {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        self.handle
            .poll(app, "Failed to Repair Mesh")
            .into_poll_result(|result| {
                let platform = app.project.slice_config.platform_size;
                let Some(model) = app.project.model(self.model) else {
                    return PollResult::complete();
                };

                let has_defects = result.any_defects();
                has_defects.then(|| model.replace_mesh(result.mesh, None, &platform));
                app.popup.open(Popup::new("Repair Report", move |_app, ui| {
                    let mut close = false;
                    if has_defects {
                        ui.label("Mesh repaired successfully, the following defects were found:");

                        ui.add_space(8.0);
                        grid("repair").show(ui, |ui| {
                            for (name, value) in [
                                ("Unwelded Vertices", result.unwelded_vertices),
                                ("Degenerative Faces", result.degenerative_faces),
                                ("Repeated Faces", result.repeated_faces),
                                ("Holes", result.holes),
                                ("Flipped Winding", result.flipped_winding),
                            ]
                            .into_iter()
                            .filter(|(_, v)| *v > 0)
                            {
                                ui.label(name);
                                ui.horizontal(|ui| {
                                    ui.label(value.to_string());
                                    ui.take_available_width();
                                });
                                ui.end_row();
                            }
                        });
                    } else {
                        ui.label("No defects found.");
                    }

                    ui.add_space(8.0);

                    ui.add_space(5.0);
                    ui.centered_and_justified(|ui| {
                        close = ui.button("Close").clicked();
                    });
                    close
                }));

                PollResult::complete()
                    .with_task(MeshDefective::new(model))
                    .with_task(BuildAccelerationStructures::new(model))
            })
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Mesh Repair".into(),
            details: None,
            progress: self.progress.progress(),
        })
    }
}
