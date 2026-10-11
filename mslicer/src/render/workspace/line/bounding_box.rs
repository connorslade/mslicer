use std::iter;

use itertools::Itertools;
use nalgebra::Vector3;
use slicer::mesh::Bounds3D;

use crate::{
    core::App,
    render::workspace::line::{LastModels, Line, LineGenerator},
};

pub struct BoundingBoxDispatch {
    last_models: LastModels,
    last_enabled: bool,

    cached_lines: Vec<Line>,
}

impl BoundingBoxDispatch {
    pub fn new() -> Self {
        Self {
            last_models: LastModels::empty(),
            last_enabled: false,

            cached_lines: Vec::new(),
        }
    }
}

impl LineGenerator for BoundingBoxDispatch {
    fn generate_lines(&mut self, app: &mut App) {
        let enabled = app.config.render.bounding_box;
        let models = app.project.models.iter().filter(|x| x.hidden);
        if self.last_models.update(models) || enabled ^ self.last_enabled {
            self.cached_lines.clear();

            if enabled {
                for model in app.project.models.iter().filter(|x| !x.hidden) {
                    let Bounds3D { min, max } = model.mesh.bounds();

                    let bottom = [
                        Vector3::new(min.x, min.y, min.z),
                        Vector3::new(max.x, min.y, min.z),
                        Vector3::new(max.x, max.y, min.z),
                        Vector3::new(min.x, max.y, min.z),
                    ];
                    let top = [
                        Vector3::new(min.x, min.y, max.z),
                        Vector3::new(max.x, min.y, max.z),
                        Vector3::new(max.x, max.y, max.z),
                        Vector3::new(min.x, max.y, max.z),
                    ];

                    let bottom_lines = bottom.iter().chain(iter::once(&bottom[0])).tuple_windows();
                    let top_lines = top.iter().chain(iter::once(&top[0])).tuple_windows();
                    let connection_lines = top.iter().zip(bottom.iter());

                    self.cached_lines.extend(
                        (bottom_lines.chain(top_lines).chain(connection_lines))
                            .map(|(a, b)| Line::new(*a, *b).color(Vector3::new(1.0, 0.5, 1.0))),
                    );
                }
            }
        }
    }

    fn lines(&self) -> &[Line] {
        &self.cached_lines
    }
}
