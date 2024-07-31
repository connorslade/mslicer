use eframe::Theme;
use nalgebra::Vector3;

use crate::render::pipelines::solid_line::Line;

pub struct BuildPlateDispatch {
    last_bed_size: Vector3<f32>,
    last_grid_size: f32,
    last_theme: Theme,

    cached_lines: Vec<Line>,
}

impl BuildPlateDispatch {
    pub fn new() -> Self {
        Self {
            last_bed_size: Vector3::zeros(),
            last_grid_size: 0.0,
            last_theme: Theme::Dark,

            cached_lines: Vec::new(),
        }
    }

    pub fn generate_lines(&mut self, bed_size: Vector3<f32>, grid_size: f32, theme: Theme) -> bool {
        if bed_size != self.last_bed_size
            || grid_size != self.last_grid_size
            || theme != self.last_theme
        {
            self.last_bed_size = bed_size;
            self.last_grid_size = grid_size;
            self.last_theme = theme;
            self.cached_lines = generate_mesh(bed_size, grid_size, theme);
            return true;
        }

        false
    }

    pub fn lines(&self) -> &[Line] {
        &self.cached_lines
    }
}

fn generate_mesh(bed_size: Vector3<f32>, grid_size: f32, theme: Theme) -> Vec<Line> {
    let (a, b) = (bed_size / 2.0, -bed_size / 2.0);
    let z = bed_size.z;

    let color = match theme {
        Theme::Dark => Vector3::new(1.0, 1.0, 1.0),
        Theme::Light => Vector3::new(0.0, 0.0, 0.0),
    };

    let line = |start, end| Line::new(start, end).color(color);

    let mut lines = vec![
        // Bottom plane
        line(Vector3::new(a.x, a.y, 0.0), Vector3::new(b.x, a.y, 0.0)),
        line(Vector3::new(a.x, b.y, 0.0), Vector3::new(b.x, b.y, 0.0)),
        line(Vector3::new(a.x, a.y, 0.0), Vector3::new(a.x, b.y, 0.0)),
        line(Vector3::new(b.x, a.y, 0.0), Vector3::new(b.x, b.y, 0.0)),
        // Top plane
        line(Vector3::new(a.x, a.y, z), Vector3::new(b.x, a.y, z)),
        line(Vector3::new(a.x, b.y, z), Vector3::new(b.x, b.y, z)),
        line(Vector3::new(a.x, a.y, z), Vector3::new(a.x, b.y, z)),
        line(Vector3::new(b.x, a.y, z), Vector3::new(b.x, b.y, z)),
        // Vertical lines
        line(Vector3::new(a.x, a.y, 0.0), Vector3::new(a.x, a.y, z)),
        line(Vector3::new(b.x, a.y, 0.0), Vector3::new(b.x, a.y, z)),
        line(Vector3::new(a.x, b.y, 0.0), Vector3::new(a.x, b.y, z)),
        line(Vector3::new(b.x, b.y, 0.0), Vector3::new(b.x, b.y, z)),
    ];

    // Grid on bottom plane
    for x in 0..(bed_size.x / grid_size).ceil() as i32 {
        let x = x as f32 * grid_size + b.x;
        lines.push(line(Vector3::new(x, a.y, 0.0), Vector3::new(x, b.y, 0.0)));
    }

    for y in 0..(bed_size.y / grid_size).ceil() as i32 {
        let y = y as f32 * grid_size + b.y;
        lines.push(line(Vector3::new(a.x, y, 0.0), Vector3::new(b.x, y, 0.0)));
    }

    lines
}
