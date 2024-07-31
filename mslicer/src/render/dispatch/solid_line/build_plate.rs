use nalgebra::Vector3;

use crate::render::pipelines::solid_line::Line;

pub struct BuildPlateDispatch {
    last_bed_size: Vector3<f32>,
    last_grid_size: f32,

    cached_lines: Vec<Line>,
}

impl BuildPlateDispatch {
    pub fn new() -> Self {
        Self {
            last_bed_size: Vector3::zeros(),
            last_grid_size: 0.0,

            cached_lines: Vec::new(),
        }
    }

    pub fn generate_lines(&mut self, bed_size: Vector3<f32>, grid_size: f32) -> bool {
        if bed_size != self.last_bed_size || grid_size != self.last_grid_size {
            self.last_bed_size = bed_size;
            self.last_grid_size = grid_size;
            self.cached_lines = generate_mesh(bed_size, grid_size);
            return true;
        }

        false
    }

    pub fn lines(&self) -> &[Line] {
        &self.cached_lines
    }
}

fn generate_mesh(bed_size: Vector3<f32>, grid_size: f32) -> Vec<Line> {
    let (a, b) = (bed_size / 2.0, -bed_size / 2.0);
    let z = bed_size.z;

    let mut lines = vec![
        // Bottom plane
        Line::new(Vector3::new(a.x, a.y, 0.0), Vector3::new(b.x, a.y, 0.0)),
        Line::new(Vector3::new(a.x, b.y, 0.0), Vector3::new(b.x, b.y, 0.0)),
        Line::new(Vector3::new(a.x, a.y, 0.0), Vector3::new(a.x, b.y, 0.0)),
        Line::new(Vector3::new(b.x, a.y, 0.0), Vector3::new(b.x, b.y, 0.0)),
        // Top plane
        Line::new(Vector3::new(a.x, a.y, z), Vector3::new(b.x, a.y, z)),
        Line::new(Vector3::new(a.x, b.y, z), Vector3::new(b.x, b.y, z)),
        Line::new(Vector3::new(a.x, a.y, z), Vector3::new(a.x, b.y, z)),
        Line::new(Vector3::new(b.x, a.y, z), Vector3::new(b.x, b.y, z)),
        // Vertical lines
        Line::new(Vector3::new(a.x, a.y, 0.0), Vector3::new(a.x, a.y, z)),
        Line::new(Vector3::new(b.x, a.y, 0.0), Vector3::new(b.x, a.y, z)),
        Line::new(Vector3::new(a.x, b.y, 0.0), Vector3::new(a.x, b.y, z)),
        Line::new(Vector3::new(b.x, b.y, 0.0), Vector3::new(b.x, b.y, z)),
    ];

    // Grid on bottom plane
    for x in 0..(bed_size.x / grid_size).ceil() as i32 {
        let x = x as f32 * grid_size + b.x;
        lines.push(Line::new(
            Vector3::new(x, a.y, 0.0),
            Vector3::new(x, b.y, 0.0),
        ));
    }

    for y in 0..(bed_size.y / grid_size).ceil() as i32 {
        let y = y as f32 * grid_size + b.y;
        lines.push(Line::new(
            Vector3::new(a.x, y, 0.0),
            Vector3::new(b.x, y, 0.0),
        ));
    }

    lines
}
