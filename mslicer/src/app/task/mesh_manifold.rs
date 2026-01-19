use std::{
    mem,
    thread::{self, JoinHandle},
};

use clone_macro::clone;
use egui::Context;

use crate::app::{
    App,
    model::{MeshWarnings, Model},
    task::Task,
};

pub struct MeshManifold {
    mesh: u32,
    join: Option<JoinHandle<bool>>,
}

impl MeshManifold {
    pub fn new(mesh: &Model) -> Self {
        Self {
            mesh: mesh.id,
            join: Some(thread::spawn(clone!([{ mesh.mesh } as model], move || {
                model.is_manifold()
            }))),
        }
    }
}

impl Task for MeshManifold {
    fn poll(&mut self, app: &mut App, _ctx: &Context) -> bool {
        if self.join.as_ref().unwrap().is_finished() {
            let result = mem::take(&mut self.join).unwrap().join().unwrap();
            if let Some(model) = app.project.models.iter_mut().find(|x| x.id == self.mesh) {
                model.warnings.set(MeshWarnings::NonManifold, !result);
            }

            true
        } else {
            false
        }
    }
}
