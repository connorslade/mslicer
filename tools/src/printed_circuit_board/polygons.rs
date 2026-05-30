use std::{f64::consts::TAU, fs::File, io::Write, path::PathBuf};

use nalgebra::Vector2;
use svgwriter::{
    Data, Graphic,
    tags::{Path, TagWithPresentationAttributes as _},
};

pub struct Polygons {
    pub polygons: Vec<Vec<Vector2<f64>>>,
}

const PRECISION: usize = 16;

impl Polygons {
    pub fn new() -> Self {
        Self {
            polygons: Vec::new(),
        }
    }

    pub fn trace(&mut self, path: Vec<Vector2<f64>>, thickness: Option<f64>) {
        if let Some(thickness) = thickness {
            self.circle(*path.first().unwrap(), thickness / 2.0);
            self.circle(*path.last().unwrap(), thickness / 2.0);
            self.polygons.push(close_path(path, thickness));
        } else {
            self.polygons.push(path);
        }
    }

    pub fn circle(&mut self, center: Vector2<f64>, r: f64) {
        let points = ((r * PRECISION as f64).ceil() as usize).max(PRECISION);

        let mut circle = Vec::with_capacity(points);
        for i in 0..points {
            let f = i as f64 / points as f64 * TAU;
            circle.push(center + Vector2::new(f.cos(), f.sin()) * r);
        }
        self.polygons.push(circle);
    }

    pub fn rect(&mut self, [min, max]: [Vector2<f64>; 2]) {
        self.polygons.push(vec![
            min,
            Vector2::new(min.x, max.y),
            max,
            Vector2::new(max.x, min.y),
        ]);
    }

    pub fn rounded_rect(&mut self, points: [Vector2<f64>; 4], radius: f64) {
        let mut out = Vec::new();
        for i in 0..4 {
            let p = points[i];
            self.circle(p, radius);

            let [a, b] = [p - points[(i + 3) % 4], points[(i + 1) % 4] - p]
                .map(|x: Vector2<f64>| x.normalize() * radius);

            out.push(p - b);
            out.push(p + a);
        }
        self.polygons.push(out);
    }

    pub fn nonuniform_scale_mut(&mut self, scale: Vector2<f64>) {
        for polygon in self.polygons.iter_mut() {
            for point in polygon.iter_mut() {
                point.x *= scale.x;
                point.y *= scale.y;
            }
        }
    }

    pub fn bounds(&self) -> [Vector2<f64>; 2] {
        let [mut min, mut max] = [f64::INFINITY, f64::NEG_INFINITY].map(Vector2::repeat);
        for polygon in self.polygons.iter() {
            for point in polygon {
                min.x = min.x.min(point.x);
                min.y = min.y.min(point.y);
                max.x = max.x.max(point.x);
                max.y = max.y.max(point.y);
            }
        }

        [min, max]
    }

    pub fn write_svg(&self, path: PathBuf) {
        let [min, max] = self.bounds();
        let [width, height] = [max.x - min.x, max.y - min.y];

        let mut svg = Graphic::new();
        svg.set_width(width as i32);
        svg.set_height(height as i32);
        svg.set_view_box(format!("{} {} {width} {height}", min.x, min.y));

        for poly in self.polygons.iter() {
            let mut data = Data::new();
            data.move_to(poly[0].x, poly[0].y);
            for point in poly.iter().skip(1) {
                data.line_to(point.x, point.y);
            }
            data.close();

            svg.push(
                Path::new()
                    .with_d(data)
                    .with_fill("#000000")
                    .with_stroke("#000000")
                    .with_stroke_width(0.01),
            );
        }

        File::create(path)
            .unwrap()
            .write_all(svg.to_string().as_bytes())
            .unwrap();
    }
}

fn close_path(path: Vec<Vector2<f64>>, path_thickness: f64) -> Vec<Vector2<f64>> {
    let half_thickness = path_thickness / 2.0;
    let mut out = vec![Vector2::zeros(); path.len() * 2];

    for i in 0..path.len() {
        let direction = if i == 0 {
            path[1] - path[0]
        } else if i + 1 == path.len() {
            path[path.len() - 1] - path[path.len() - 2]
        } else {
            path[i + 1] - path[i - 1]
        }
        .normalize();

        let normal = Vector2::new(-direction.y, direction.x).scale(half_thickness);
        out[i] = path[i] + normal;
        out[path.len() * 2 - i - 1] = path[i] - normal;
    }

    out
}
