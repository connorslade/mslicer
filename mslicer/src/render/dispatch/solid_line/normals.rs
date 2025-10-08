use std::sync::Arc;

use nalgebra::{Matrix4, Vector3};
use parking_lot::RwLock;

use crate::render::{
    pipelines::solid_line::Line, rendered_mesh::RenderedMesh, workspace::WorkspaceRenderCallback,
};

use super::LineDispatch;

type Models = Arc<RwLock<Vec<RenderedMesh>>>;

pub struct NormalsDispatch {
    last_models: Vec<u32>,
    last_transforms: Vec<Matrix4<f32>>,
    last_normals: bool,

    cached_lines: Vec<Line>,
}

impl NormalsDispatch {
    pub fn new() -> Self {
        Self {
            last_models: Vec::new(),
            last_transforms: Vec::new(),
            last_normals: false,

            cached_lines: Vec::new(),
        }
    }
}

impl LineDispatch for NormalsDispatch {
    fn generate_lines(&mut self, resources: &WorkspaceRenderCallback) -> bool {
        let show_normals = resources.config.show_normals;
        if !show_normals && show_normals == self.last_normals {
            return false;
        }

        let models = resources.models.read();
        let ids = models
            .iter()
            .filter(|x| x.hidden)
            .map(|x| x.id)
            .collect::<Vec<_>>();
        let transforms = models
            .iter()
            .map(|x| *x.mesh.transformation_matrix())
            .collect::<Vec<_>>();

        if ids != self.last_models
            || transforms != self.last_transforms
            || show_normals != self.last_normals
        {
            self.last_models = ids;
            self.last_transforms = transforms;
            self.last_normals = show_normals;

            if show_normals {
                self.cached_lines = generate_normals(resources.models.clone());
            } else {
                self.cached_lines = Vec::new();
            }

            return true;
        }

        false
    }

    fn lines(&self) -> &[Line] {
        &self.cached_lines
    }
}

fn generate_normals(models: Models) -> Vec<Line> {
    let color = Vector3::new(0.5, 0.5, 1.0);
    let mut lines = Vec::new();

    for model in models.read().iter().filter(|x| !x.hidden) {
        let (face, vertices, normals) = (
            model.mesh.faces(),
            model.mesh.vertices(),
            model.mesh.normals(),
        );

        for (&face, normal) in face.iter().zip(normals.iter()) {
            let center = face
                .iter()
                .map(|&idx| vertices[idx as usize])
                .sum::<Vector3<f32>>()
                / 3.0;
            let center = model.mesh.transform(&center);
            let normal = model.mesh.transform_normal(normal);

            lines.push(Line::new(center, center + normal * 0.2).color(color));
        }
    }

    lines
}
