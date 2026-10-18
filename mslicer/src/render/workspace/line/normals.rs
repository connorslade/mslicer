use nalgebra::Vector3;

use crate::{
    core::{App, config::render::DebugRender, project::model::Model},
    render::workspace::line::{LastModels, Line, LineGenerator},
};

pub struct NormalsDispatch {
    last_models: LastModels,
    last_config: DebugRender,

    cached_lines: Vec<Line>,
}

impl NormalsDispatch {
    pub fn new() -> Self {
        Self {
            last_models: LastModels::empty(),
            last_config: DebugRender::new(),

            cached_lines: Vec::new(),
        }
    }
}

impl LineGenerator for NormalsDispatch {
    fn generate_lines(&mut self, app: &mut App) {
        let config = &app.config.render.normals;
        let models = app.project.models.iter().filter(|x| !x.hidden);
        if self.last_models.update(models) || config != &self.last_config {
            self.cached_lines.clear();
            self.last_config = config.clone();

            if config.enabled {
                self.cached_lines = generate_normals(&app.project.models);
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
