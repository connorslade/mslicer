use std::f32::consts::PI;

use nalgebra::Vector3;

use crate::mesh::Mesh;

pub struct MeshBuilder {
    vertices: Vec<Vector3<f32>>,
    normals: Vec<Vector3<f32>>,
    faces: Vec<[u32; 3]>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            normals: Vec::new(),
            faces: Vec::new(),
        }
    }

    pub fn add_vertex(&mut self, vertex: Vector3<f32>) -> u32 {
        self.vertices.push(vertex);
        (self.vertices.len() - 1) as u32
    }

    pub fn add_face(&mut self, face: [u32; 3], normal: Vector3<f32>) {
        self.faces.push(face);
        self.normals.push(normal);
    }

    pub fn add_quad(&mut self, quad: [u32; 4], normal: Vector3<f32>) {
        self.add_face([quad[0], quad[1], quad[2]], normal);
        self.add_face([quad[2], quad[1], quad[3]], normal);
    }

    pub fn build(self) -> Mesh {
        Mesh::new_uncentred(self.vertices, self.faces, self.normals)
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
            let angle = 2.0 * PI * (i as f32) / (precision as f32);
            let normal = Vector3::new(angle.sin(), angle.cos(), 0.0);

            let top = self.add_vertex(top + normal * top_radius);
            let bottom = self.add_vertex(bottom + normal * bottom_radius);

            if let Some((last_top, last_bottom)) = last {
                self.add_quad([last_top, last_bottom, top, bottom], normal);
                self.add_face([last_top, top, top_center], Vector3::z());
                self.add_face([last_bottom, bottom_center, bottom], -Vector3::z());
            }

            last = Some((top, bottom));
            if fist.is_none() {
                fist = Some((top, bottom, normal));
            }
        }

        if let Some((first_top, first_bottom, normal)) = fist {
            if let Some((last_top, last_bottom)) = last {
                self.add_quad([last_top, last_bottom, first_top, first_bottom], normal);
                self.add_face([last_top, first_top, top_center], Vector3::z());
                self.add_face([last_bottom, bottom_center, first_bottom], -Vector3::z());
            }
        }
    }
}

impl Default for MeshBuilder {
    fn default() -> Self {
        Self::new()
    }
}
