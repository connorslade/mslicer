use std::f32::consts::{PI, TAU};

use nalgebra::Vector3;

use crate::mesh::Mesh;

pub struct MeshBuilder {
    vertices: Vec<Vector3<f32>>,
    faces: Vec<[u32; 3]>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            faces: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.faces.is_empty()
    }

    pub fn add_vertex(&mut self, vertex: Vector3<f32>) -> u32 {
        self.vertices.push(vertex);
        (self.vertices.len() - 1) as u32
    }

    pub fn add_face(&mut self, face: [u32; 3]) {
        self.faces.push(face);
    }

    pub fn add_quad(&mut self, quad: [u32; 4]) {
        self.add_face([quad[0], quad[2], quad[1]]);
        self.add_face([quad[1], quad[2], quad[3]]);
    }

    pub fn build(self) -> Mesh {
        Mesh::new_uncentred(self.vertices, self.faces)
    }
}

impl MeshBuilder {
    pub fn add_vertical_cylinder(
        &mut self,
        bottom: Vector3<f32>,
        height: f32,
        radii: (f32, f32),
        precision: u32,
    ) {
        let top = bottom + Vector3::z() * height;
        self.add_cylinder((bottom, top), radii, precision);
    }

    pub fn add_cylinder(
        &mut self,
        (a, b): (Vector3<f32>, Vector3<f32>),
        (a_radius, b_radius): (f32, f32),
        precision: u32,
    ) {
        let [u, v] = orthogonal_basis((a - b).normalize());

        let bottom_center = self.add_vertex(a);
        let top_center = self.add_vertex(b);

        let mut first = None;
        let mut last = None;
        for i in 0..(precision * 2) {
            let angle = i as f32 / precision as f32 * PI;
            let normal = u * angle.sin() + v * angle.cos();

            let top = self.add_vertex(b + normal * b_radius);
            let bottom = self.add_vertex(a + normal * a_radius);

            if let Some((last_top, last_bottom)) = last {
                self.add_quad([last_bottom, last_top, bottom, top]);
                self.add_face([top_center, last_top, top]);
                self.add_face([bottom, last_bottom, bottom_center]);
            }

            last = Some((top, bottom));
            first.is_none().then(|| first = last);
        }

        if let Some((last_top, last_bottom)) = last
            && let Some((first_top, first_bottom)) = first
        {
            self.add_quad([last_bottom, last_top, first_bottom, first_top]);
            self.add_face([top_center, last_top, first_top]);
            self.add_face([first_bottom, last_bottom, bottom_center]);
        }
    }

    /// Adds a sphere mesh with the specified number of vertices along the pitch and azimuth.
    pub fn add_sphere(&mut self, center: Vector3<f32>, radius: f32, precision: u32) {
        let north = self.add_vertex(center + Vector3::z() * radius);
        let south = self.add_vertex(center - Vector3::z() * radius);
        let first = south + 1;

        for i_phi in 1..precision {
            let phi = i_phi as f32 / precision as f32 * PI;
            for i_theta in 0..precision {
                let theta = i_theta as f32 / precision as f32 * TAU;
                let dir = Vector3::new(phi.sin() * theta.cos(), phi.sin() * theta.sin(), phi.cos());
                self.add_vertex(center + dir * radius);
            }
        }

        let last = first + precision * (precision - 2);
        for i in 0..precision {
            let j = (i + 1) % precision;
            self.add_face([north, first + i, first + j]);
            self.add_face([south, last + j, last + i]);
        }

        for ring in 0..(precision - 2) {
            let curr = first + precision * ring;
            let next = first + precision * (ring + 1);
            for i in 0..precision {
                let j = (i + 1) % precision;
                self.add_quad([curr + i, curr + j, next + i, next + j]);
            }
        }
    }
}

// Hughes Moeller method. Input vector should be normalized.
fn orthogonal_basis(n: Vector3<f32>) -> [Vector3<f32>; 2] {
    let basis = if n.x.abs() > n.z.abs() {
        Vector3::new(-n.y, n.x, 0.0)
    } else {
        Vector3::new(0.0, -n.z, n.y)
    }
    .normalize();
    [basis, n.cross(&basis)]
}

impl Default for MeshBuilder {
    fn default() -> Self {
        Self::new()
    }
}
