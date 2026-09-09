use std::f64::consts::{PI, TAU};

use nalgebra::Vector2;
use svg::{
    Document,
    node::element::{Path, path::Data},
};

use crate::misc::bounds::Bounds2D;

pub struct Polygons {
    pub polygons: Vec<Vec<Vector2<f64>>>,
    pub bounds: Bounds2D<f64>,

    pub mode: Mode,
    pub sagitta: f64,
}

#[derive(Copy, Clone)]
pub struct Mode {
    pub polygon: bool,
    pub bounds: bool,
}

impl Polygons {
    pub fn new() -> Self {
        Self {
            polygons: Vec::new(),
            bounds: Bounds2D::<f64>::EMPTY,

            mode: Mode {
                polygon: true,
                bounds: true,
            },
            sagitta: 0.01, // 10μm
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn set_max_circle_error(&mut self, sagitta: f64) {
        self.sagitta = sagitta;
    }

    fn polygon(&mut self, points: Vec<Vector2<f64>>) {
        if self.mode.bounds {
            for point in points.iter() {
                self.bounds
                    .include_bound_mut(Bounds2D::new_point(point.cast()));
            }
        }

        self.mode.polygon.then(|| self.polygons.push(points));
    }

    pub fn trace(&mut self, path: Vec<Vector2<f64>>, thickness: Option<f64>) {
        if let Some(thickness) = thickness {
            let (first, last) = (*path.first().unwrap(), *path.last().unwrap());
            if first != last {
                self.circle(first, thickness / 2.0);
                self.circle(last, thickness / 2.0);
                self.polygon(inflate_path(path, thickness));
            } else {
                self.polygon(inflate_closed_path(path, thickness));
            }
        } else {
            self.polygon(path);
        }
    }

    pub fn circle(&mut self, center: Vector2<f64>, r: f64) {
        // Reference: https://en.wikipedia.org/wiki/Sagitta_(geometry)
        // self.sagitta = r (1 - cos θ/2)
        // 2 cos⁻¹(-(self.sagitta / r - 1)) = θ
        // n = π / cos⁻¹(1 - self.sagitta / r)
        let points = ((PI / (1.0 - self.sagitta / r).acos()).ceil() as usize).max(3);

        let mut circle = Vec::with_capacity(points);
        for i in 0..points {
            let f = i as f64 / points as f64 * TAU;
            circle.push(center + Vector2::new(f.cos(), f.sin()) * r);
        }
        self.polygon(circle);
    }

    pub fn rect(&mut self, [min, max]: [Vector2<f64>; 2]) {
        self.polygon(vec![
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
        self.polygon(out);
    }

    /// Note that bounds are not updated to reflect the transformation.
    pub fn nonuniform_scale_mut(&mut self, scale: Vector2<f64>) {
        for polygon in self.polygons.iter_mut() {
            for point in polygon.iter_mut() {
                point.x *= scale.x;
                point.y *= scale.y;
            }
        }
    }

    /// Note that bounds are not updated to reflect the transformation.
    pub fn translate_mut(&mut self, transform: Vector2<f64>) {
        for polygon in self.polygons.iter_mut() {
            for point in polygon.iter_mut() {
                point.x += transform.x;
                point.y += transform.y;
            }
        }
    }

    pub fn svg(&self) -> String {
        let size = self.bounds.size();

        let view_box = (self.bounds.min.x, self.bounds.min.y, size.x, size.y);
        let mut svg = Document::new()
            .set("viewBox", view_box)
            .set("width", size.x as u32)
            .set("height", size.y as i32);

        for poly in self.polygons.iter() {
            let mut data = Data::new().move_to((poly[0].x, poly[0].y));
            for point in poly.iter().skip(1) {
                data = data.line_to((point.x, point.y));
            }

            let path = Path::new()
                .set("d", data.close())
                .set("fill", "#000000")
                .set("stroke", "#000000")
                .set("stroke-width", 0.01);
            svg = svg.add(path);
        }

        svg.to_string()
    }
}

impl Default for Polygons {
    fn default() -> Self {
        Self::new()
    }
}

fn inflate_path(path: Vec<Vector2<f64>>, path_thickness: f64) -> Vec<Vector2<f64>> {
    let half_thickness = path_thickness / 2.0;
    let mut out = vec![Vector2::zeros(); path.len() * 2];

    for (i, this) in path.iter().enumerate() {
        let normal = if i == 0 {
            rotate_ccw((path[1] - this).normalize()) * half_thickness
        } else if i + 1 == path.len() {
            rotate_ccw((this - path[i - 1]).normalize()) * half_thickness
        } else {
            // If the current point it not an endpoint, offset it halfway
            // between the normals of each connected segment
            let bisect = (path[i + 1] - this).normalize() + (this - path[i - 1]).normalize();
            rotate_ccw(bisect.normalize()) * path_thickness / bisect.magnitude()
        };

        out[i] = path[i] + normal;
        out[path.len() * 2 - i - 1] = path[i] - normal;
    }

    out
}

fn inflate_closed_path(path: Vec<Vector2<f64>>, path_thickness: f64) -> Vec<Vector2<f64>> {
    let mut out = vec![Vector2::zeros(); path.len() * 2];
    let path = &path[1..]; // ignore the duplicated vert

    for (i, this) in path.iter().enumerate() {
        // Wrap around the endpoints
        let [prev, next] = if i == 0 {
            [path[path.len() - 1], path[i + 1]]
        } else if i + 1 == path.len() {
            [path[path.len() - 2], path[0]]
        } else {
            [path[i - 1], path[i + 1]]
        };

        let bisect = (next - this).normalize() + (this - prev).normalize();
        let normal = rotate_ccw(bisect.normalize()) * path_thickness / bisect.magnitude();

        out[i] = path[i] + normal;
        out[(path.len() + 1) * 2 - i - 1] = path[i] - normal;
    }

    // duplicate the firsts point to the end of the list to close the shape
    out[path.len()] = out[0];
    out[path.len() + 1] = out[(path.len() + 1) * 2 - 1];

    out
}

fn rotate_ccw(v: Vector2<f64>) -> Vector2<f64> {
    Vector2::new(-v.y, v.x)
}
