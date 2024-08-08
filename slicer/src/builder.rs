use std::f32::consts::PI;

use anyhow::Result;
use nalgebra::Vector3;
use stl_io::Triangle;

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
        self.add_face([quad[1], quad[2], quad[3]], normal);
    }

    pub fn build(self) -> Mesh {
        Mesh::new(self.vertices, self.faces, self.normals)
    }

    pub fn save_stl(&self, path: &str) -> Result<()> {
        let mut stl = Vec::new();

        for (face, normal) in self.faces.iter().zip(self.normals.iter()) {
            let (a, b, c) = (
                self.vertices[face[0] as usize],
                self.vertices[face[1] as usize],
                self.vertices[face[2] as usize],
            );

            let into_stl = |v: Vector3<f32>| stl_io::Vertex::new([v.x, v.y, v.z]);
            stl.push(Triangle {
                normal: into_stl(*normal),
                vertices: [into_stl(a), into_stl(b), into_stl(c)],
            });
        }

        let mut file = std::fs::File::create(path)?;
        stl_io::write_stl(&mut file, stl.iter())?;
        Ok(())
    }
}

impl MeshBuilder {
    pub fn add_vertical_cylinder(
        &mut self,
        bottom: Vector3<f32>,
        height: f32,
        radius: f32,
        precision: u32,
    ) {
        let top = bottom + Vector3::new(0.0, 0.0, height);
        let bottom_center = self.add_vertex(bottom);
        let top_center = self.add_vertex(top);

        let mut last = None;
        let mut fist = None;
        for i in 0..precision {
            let angle = 2.0 * PI * (i as f32) / (precision as f32);
            let normal = Vector3::new(angle.cos(), angle.sin(), 0.0);
            let offset = normal * radius;

            let top = self.add_vertex(top + offset);
            let bottom = self.add_vertex(bottom + offset);

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
