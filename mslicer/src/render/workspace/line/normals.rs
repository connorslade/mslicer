use nalgebra::{Matrix4, Vector3};

use crate::{
    app::{App, project::model::Model},
    render::workspace::line::{Line, LineGenerator},
};

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

impl LineGenerator for NormalsDispatch {
    fn generate_lines(&mut self, app: &mut App) {
        let show_normals = app.config.show_normals;
        if !show_normals && show_normals == self.last_normals {
            return;
        }

        let ids = (app.project.models.iter())
            .filter(|x| x.hidden)
            .map(|x| x.id)
            .collect::<Vec<_>>();
        let transforms = (app.project.models.iter())
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
                self.cached_lines = generate_normals(&app.project.models);
            } else {
                self.cached_lines = Vec::new();
            }
        }
    }

    fn lines(&self) -> &[Line] {
        &self.cached_lines
    }
}

fn generate_normals(models: &[Model]) -> Vec<Line> {
    let color = Vector3::new(0.5, 0.5, 1.0);
    let mut lines = Vec::new();

    for model in models.iter().filter(|x| !x.hidden) {
        let (face, vertices) = (model.mesh.faces(), model.mesh.vertices());

        for (idx, &face) in face.iter().enumerate() {
            let center = (face.iter())
                .map(|&idx| vertices[idx as usize])
                .sum::<Vector3<f32>>()
                / 3.0;
            let center = model.mesh.transform(&center);
            let normal = model.mesh.transform_normal(&model.mesh.normal(idx));

            lines.push(Line::new(center, center + normal * 0.2).color(color));
        }
    }

    lines
}
