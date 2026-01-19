use nalgebra::Vector4;

use crate::{
    app::App,
    render::workspace::point::{Point, PointGenerator},
};

pub struct OverhangPointDispatch {
    cached_points: Vec<Point>,
}

impl OverhangPointDispatch {
    pub fn new() -> Self {
        Self {
            cached_points: Vec::new(),
        }
    }
}

impl PointGenerator for OverhangPointDispatch {
    fn generate_points(&mut self, app: &mut App) {
        self.cached_points.clear();

        for model in app.project.models.iter() {
            let Some(overhangs) = &model.overhangs else {
                continue;
            };

            let verts = model.mesh.vertices();
            for overhang in overhangs {
                let position = model.mesh.transform(&verts[*overhang as usize]);
                self.cached_points.push(Point {
                    position,
                    radius: 0.5,
                    color: Vector4::new(1.0, 1.0, 0.0, 0.25),
                });
            }
        }
    }

    fn points(&self) -> &[Point] {
        &self.cached_points
    }
}
