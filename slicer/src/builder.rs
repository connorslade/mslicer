use std::f32::consts::TAU;

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

    pub fn add_vertex(&mut self, vertex: Vector3<f32>) -> u32 {
        self.vertices.push(vertex);
        (self.vertices.len() - 1) as u32
    }

    pub fn add_face(&mut self, face: [u32; 3]) {
        self.faces.push(face);
    }

    pub fn add_quad(&mut self, quad: [u32; 4]) {
        self.add_face([quad[0], quad[1], quad[2]]);
        self.add_face([quad[2], quad[1], quad[3]]);
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
        (bottom_radius, top_radius): (f32, f32),
        precision: u32,
    ) {
        let top = bottom + Vector3::new(0.0, 0.0, height);
        let bottom_center = self.add_vertex(bottom);
        let top_center = self.add_vertex(top);

        let mut last = None;
        let mut fist = None;
        for i in 0..precision {
            let angle = TAU * (i as f32) / (precision as f32);
            let normal = Vector3::new(angle.sin(), angle.cos(), 0.0);

            let top = self.add_vertex(top + normal * top_radius);
            let bottom = self.add_vertex(bottom + normal * bottom_radius);

            if let Some((last_top, last_bottom)) = last {
                self.add_quad([last_bottom, last_top, bottom, top]);
                self.add_face([top, last_top, top_center]);
                self.add_face([bottom_center, last_bottom, bottom]);
            }

            last = Some((top, bottom));
            if fist.is_none() {
                fist = Some((top, bottom));
            }
        }

        if let Some((first_top, first_bottom)) = fist {
            if let Some((last_top, last_bottom)) = last {
                self.add_quad([last_bottom, last_top, first_bottom, first_top]);
                self.add_face([first_top, last_top, top_center]);
                self.add_face([bottom_center, last_bottom, first_bottom]);
            }
        }
    }
}

impl Default for MeshBuilder {
    fn default() -> Self {
        Self::new()
    }
}
