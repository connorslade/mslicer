use std::{
    io::{BufRead, Seek},
    sync::Arc,
};

use anyhow::Result;
use nalgebra::{Matrix4, Vector3};
use obj::{Obj, Position};

use crate::Pos;

/// A mesh made of vertices and triangular faces. It can be scaled, translated,
/// and rotated.
#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Arc<Box<[Pos]>>,
    pub faces: Arc<Box<[[u32; 3]]>>,

    transformation_matrix: Matrix4<f32>,
    inv_transformation_matrix: Matrix4<f32>,

    position: Pos,
    scale: Pos,
    rotation: Pos,
}

impl Mesh {
    /// Creates a new mesh from the givin vertices and faces. The
    /// transformations are all 0 by default.
    pub fn new(mut vertices: Vec<Pos>, faces: Vec<[u32; 3]>) -> Self {
        center_vertices(&mut vertices);

        Self {
            vertices: Arc::new(vertices.into_boxed_slice()),
            faces: Arc::new(faces.into_boxed_slice()),
            ..Default::default()
        }
    }

    /// Intersect the mesh with a plane with linier time complexity. You
    /// should probably use the [`crate::segments::Segments`] struct as it can
    /// massively accurate slicing of high face count triangles.
    pub fn intersect_plane(&self, height: f32) -> Vec<Pos> {
        // Point is the position of the plane and normal is the direction /
        // rotation of the plane.
        let point = self.inv_transform(&Vector3::new(0.0, 0.0, height));
        let normal = (self.inv_transformation_matrix * Vector3::z_axis().to_homogeneous()).xyz();

        let mut out = Vec::new();

        for face in self.faces.iter() {
            // Get the vertices of the face
            let v0 = self.vertices[face[0] as usize];
            let v1 = self.vertices[face[1] as usize];
            let v2 = self.vertices[face[2] as usize];

            // By subtracting the position of the plane and doting it with the
            // normal, we get a value that is positive if the point is above the
            // plane and negative if it is below. By checking if any of the line
            // segments of triangle have one point above the plane and one
            // below, we find any line segments that are intersecting with the
            // plane.
            let (a, b, c) = (
                (v0 - point).dot(&normal),
                (v1 - point).dot(&normal),
                (v2 - point).dot(&normal),
            );
            let (a_pos, b_pos, c_pos) = (a > 0.0, b > 0.0, c > 0.0);

            // Closure called when the line segment from v0 to v1 is intersecting the
            // plane. t is how far along the line the intersection is and intersection,
            // it well the point that is intersecting with the plane.
            let mut push_intersection = |a: f32, b: f32, v0: Pos, v1: Pos| {
                let (v0, v1) = (self.transform(&v0), self.transform(&v1));
                let t = a / (a - b);
                let intersection = v0 + t * (v1 - v0);
                out.push(intersection);
            };

            (a_pos ^ b_pos).then(|| push_intersection(a, b, v0, v1));
            (b_pos ^ c_pos).then(|| push_intersection(b, c, v1, v2));
            (c_pos ^ a_pos).then(|| push_intersection(c, a, v2, v0));
        }

        out
    }

    /// Updates the internal transformation matrices. This is called
    /// automatically if you use [`Mesh::set_position`], [`Mesh::set_scale`], or
    /// [`Mesh::set_rotation`], but you will need to call it manually if you use
    /// the unchecked variants of those methods.
    pub fn update_transformation_matrix(&mut self) {
        let scale = Matrix4::new_nonuniform_scaling(&self.scale);
        let rotation =
            Matrix4::from_euler_angles(self.rotation.x, self.rotation.y, self.rotation.z);
        let translation = Matrix4::new_translation(&self.position);

        self.transformation_matrix = translation * scale * rotation;
        self.inv_transformation_matrix = self.transformation_matrix.try_inverse().unwrap();
    }

    /// Transforms a point according to the models translation, scale, and rotation.
    pub fn transform(&self, pos: &Pos) -> Pos {
        (self.transformation_matrix * pos.push(1.0)).xyz()
    }

    /// Undoes the transformation of a point from the models translation, scale, and rotation.
    pub fn inv_transform(&self, pos: &Pos) -> Pos {
        (self.inv_transformation_matrix * pos.push(1.0)).xyz()
    }

    /// Get the minimum and maximum of each component of every vertex in the
    /// model. These points define the bounding box of the model.
    pub fn minmax_point(&self) -> (Pos, Pos) {
        minmax_vertices(&self.vertices, &self.transformation_matrix)
    }
}

impl Mesh {
    /// Gets the current transformation matrix of the model.
    pub fn transformation_matrix(&self) -> &Matrix4<f32> {
        &self.transformation_matrix
    }

    /// Gets the inverse of the current transformation matrix of the model.
    pub fn inv_transformation_matrix(&self) -> &Matrix4<f32> {
        &self.inv_transformation_matrix
    }

    /// Changes the position of the model, automatically updating the internal
    /// transformation matrix.
    pub fn set_position(&mut self, pos: Pos) {
        self.position = pos;
        self.update_transformation_matrix();
    }

    /// Changes the position of the model without updating the internal
    /// transformation matrix. You will need to manually call
    /// [`Mesh::update_transformation_matrix`] at some point.
    pub fn set_position_unchecked(&mut self, pos: Pos) {
        self.position = pos;
    }

    /// Gets the current position of the model.
    pub fn position(&self) -> Pos {
        self.position
    }

    /// Changes the current scale of the model, automatically updating the
    /// internal transformation matrix.
    pub fn set_scale(&mut self, scale: Pos) {
        self.scale = scale;
        self.update_transformation_matrix();
    }

    /// Changes the current scale of the model without updating the internal
    /// transformation matrix. You will need to manually call
    /// [`Mesh::update_transformation_matrix`] at some point.
    pub fn set_scale_unchecked(&mut self, scale: Pos) {
        self.scale = scale;
    }

    /// Gets the current scale of the model.
    pub fn scale(&self) -> Pos {
        self.scale
    }

    /// Changes the current rotation of the model, using [Euler
    /// angles](https://en.wikipedia.org/wiki/Euler_angles). The internal
    /// transformation matrix is automatically updated.
    pub fn set_rotation(&mut self, rotation: Pos) {
        self.rotation = rotation;
        self.update_transformation_matrix();
    }

    /// Changes the current rotation of the model (see [`Mesh::set_rotation`]),
    /// without updating the internal transformation matrix. You will need to
    /// manually call [`Mesh::update_transformation_matrix`] at some point.
    pub fn set_rotation_unchecked(&mut self, rotation: Pos) {
        self.rotation = rotation;
    }

    /// Gets the current roation of the model.
    pub fn rotation(&self) -> Pos {
        self.rotation
    }
}

/// Loads a buffer into a mesh.
/// Supported formats include:
/// - stl
/// - obj / mtl
pub fn load_mesh<T: BufRead + Seek>(reader: &mut T, format: &str) -> Result<Mesh> {
    Ok(match format {
        "stl" => {
            let model = stl_io::read_stl(reader)?;
            Mesh::new(
                model
                    .vertices
                    .iter()
                    .map(|v| Pos::new(v[0], v[1], v[2]))
                    .collect(),
                model
                    .faces
                    .iter()
                    .map(|f| {
                        [
                            f.vertices[0] as u32,
                            f.vertices[1] as u32,
                            f.vertices[2] as u32,
                        ]
                    })
                    .collect(),
            )
        }
        "obj" | "mtl" => {
            let model: Obj<Position> = obj::load_obj(reader)?;
            Mesh::new(
                model
                    .vertices
                    .iter()
                    .map(|v| Pos::new(v.position[0], v.position[1], v.position[2]))
                    .collect(),
                model
                    .indices
                    .chunks_exact(3)
                    .map(|i| [i[0] as u32, i[1] as u32, i[2] as u32])
                    .collect(),
            )
        }
        _ => return Err(anyhow::anyhow!("Unsupported format: {}", format)),
    })
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            vertices: Default::default(),
            faces: Default::default(),

            transformation_matrix: Matrix4::identity(),
            inv_transformation_matrix: Matrix4::identity(),

            position: Pos::new(0.0, 0.0, 0.0),
            scale: Pos::new(1.0, 1.0, 1.0),
            rotation: Pos::new(0.0, 0.0, 0.0),
        }
    }
}

// todo: maybe only transform min and max at end
/// Get the minimum and maximum of each component of every vertex.
/// These points define the bounding box of the model.
fn minmax_vertices(vertices: &[Pos], transform: &Matrix4<f32>) -> (Pos, Pos) {
    vertices.iter().fold(
        (
            Pos::new(f32::MAX, f32::MAX, f32::MAX),
            Pos::new(f32::MIN, f32::MIN, f32::MIN),
        ),
        |(min, max), v| {
            let v = transform * v.push(1.0);
            (
                Pos::new(min.x.min(v.x), min.y.min(v.y), min.z.min(v.z)),
                Pos::new(max.x.max(v.x), max.y.max(v.y), max.z.max(v.z)),
            )
        },
    )
}

/// Moves the model to have its origin at its centerpoint.
fn center_vertices(vertices: &mut [Pos]) {
    let (min, max) = minmax_vertices(vertices, &Matrix4::identity());

    let center = (min + max) / 2.0;
    let center = Pos::new(center.x, center.y, min.z);
    vertices.iter_mut().for_each(|v| *v -= center);
}
