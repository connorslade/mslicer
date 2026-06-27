use std::{collections::HashMap, mem};

use clone_macro::clone;
use common::{container::ArrayCluster, progress::Progress};
use slicer::mesh::Mesh;

use crate::{
    project::{Collection, model::Model},
    task::{
        BuildAccelerationStructures, MeshManifold, PollResult, Task, TaskApp, TaskStatus,
        thread::TaskThread,
    },
};

pub struct SplitBodies {
    collection: Collection,
    progress: Progress,
    handle: TaskThread<Vec<Mesh>>,
}

impl SplitBodies {
    pub fn new(model: &mut Model) -> Self {
        let mesh = model.mesh.clone();

        let collection = Collection::new(model.name.to_owned());
        model.collection = Some(collection.id);
        model.hidden = true;

        let progress = Progress::new();
        Self {
            handle: TaskThread::spawn(clone!([progress], move || {
                let mut clusters = ArrayCluster::new(mesh.vertex_count());
                progress.set_total(mesh.face_count() as u64 * 3);
                for [a, b, c] in mesh.faces() {
                    for (&a, &b) in [(a, b), (b, c), (c, a)] {
                        clusters.union(a, b);
                        progress.add_complete(1);
                    }
                }

                let mut out = Vec::new();
                for body in clusters.clusters() {
                    let mut verts = Vec::new();
                    let mut vert_cache = HashMap::<u32, u32>::new();
                    let mut faces = Vec::new();

                    let mut vert = |old: u32| {
                        if let Some(new) = vert_cache.get(&old) {
                            *new
                        } else {
                            let new = verts.len() as u32;
                            verts.push(mesh.vertices()[old as usize]);
                            vert_cache.insert(old, new);
                            new
                        }
                    };

                    // if {a, b, c} ∈ body
                    for f @ [a, ..] in mesh.faces() {
                        body.contains(a).then(|| faces.push(f.map(&mut vert)));
                    }

                    out.push(Mesh::new(verts, faces));
                }

                progress.set_finished();
                out
            })),
            collection,
            progress,
        }
    }
}

impl Task for SplitBodies {
    fn poll(&mut self, app: &mut TaskApp) -> PollResult {
        self.handle
            .poll(app, "Unexpected Error Splitting Bodies")
            .into_poll_result(|meshes| {
                let mut result = PollResult::complete();

                let collection = mem::take(&mut self.collection);
                for (i, mesh) in meshes.into_iter().enumerate() {
                    let mut model = Model::from_mesh(mesh)
                        .with_name(format!("Body {i}"))
                        .with_collection(Some(collection.id))
                        .with_random_color();
                    model.update_oob(&app.project.slice_config.platform_size);
                    result = result
                        .with_task(MeshManifold::new(&model))
                        .with_task(BuildAccelerationStructures::new(&model));
                    app.project.models.push(model);
                }

                result
            })
    }

    fn status(&self) -> Option<TaskStatus<'_>> {
        Some(TaskStatus {
            name: "Splitting Bodies".into(),
            details: None,
            progress: self.progress.progress(),
        })
    }
}
