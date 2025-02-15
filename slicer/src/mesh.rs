use std::{
    io::{BufRead, Seek},
    sync::Arc,
};

use anyhow::Result;
use nalgebra::{Matrix4, Vector3};
use obj::{Obj, Vertex};
use tracing::warn;

use crate::Pos;

/// A mesh made of vertices and triangular faces. It can be scaled, translated,
/// and rotated.
#[derive(Debug, Clone)]
pub struct Mesh {
    inner: Arc<MeshInner>,

    transformation_matrix: Matrix4<f32>,
    inv_transformation_matrix: Matrix4<f32>,

    position: Pos,
    scale: Pos,
    rotation: Pos,
}

#[derive(Debug)]
struct MeshInner {
    pub vertices: Box<[Vector3<f32>]>,
    pub faces: Box<[[u32; 3]]>,
    pub normals: Box<[Vector3<f32>]>,
}

impl Mesh {
    /// Creates a new mesh from the givin vertices and faces. The
    /// transformations are all 0 by default.
    pub fn new(mut vertices: Vec<Pos>, faces: Vec<[u32; 3]>, mut normals: Vec<Pos>) -> Self {
        center_vertices(&mut vertices);

        if normals.iter().any(|x| x.magnitude_squared() == 0.0) {
            warn!("Model has invalid normals. Recomputing.");
            normals = recompute_normals(&vertices, &faces);
        }

        Self::new_uncentred(vertices, faces, normals)
    }

    /// Creates a new mesh from the givin vertices and faces. The
    /// transformations are all 0 by default and the vertices are
    /// not centered.
    pub fn new_uncentred(vertices: Vec<Pos>, faces: Vec<[u32; 3]>, normals: Vec<Pos>) -> Self {
        Self {
            inner: Arc::new(MeshInner {
                vertices: vertices.into_boxed_slice(),
                faces: faces.into_boxed_slice(),
                normals: normals.into_boxed_slice(),
            }),
            ..Default::default()
        }
    }

    pub fn vertices(&self) -> &[Pos] {
        self.inner.vertices.as_ref()
    }

    pub fn faces(&self) -> &[[u32; 3]] {
        self.inner.faces.as_ref()
    }

    pub fn normals(&self) -> &[Pos] {
        self.inner.normals.as_ref()
    }

    pub fn face(&self, index: usize) -> &[u32; 3] {
        self.faces().get(index).unwrap()
    }

    pub fn normal(&self, index: usize) -> &Pos {
        self.normals().get(index).unwrap()
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices().len()
    }

    pub fn face_count(&self) -> usize {
        self.faces().len()
    }

    /// Makes a copy of the mesh with normals computed from the triangles
    /// directly. The copy makes this operation kinda expensive.
    pub fn recompute_normals(&mut self) {
        let normals = recompute_normals(self.vertices(), self.faces());
        self.overwrite_normals(normals)
    }

    /// Makes a copy of the mesh with normals computed from the triangles
    /// directly. The copy makes this operation kinda expensive.
    pub fn flip_normals(&mut self) {
        let normals = self.normals().iter().map(|f| -f).collect();
        self.overwrite_normals(normals);
    }

    // The alternaitve is to put normals in a separate Arc to avoid cloning all
    // verts and faces here, but its proooobly fine.
    fn overwrite_normals(&mut self, normals: Vec<Vector3<f32>>) {
        self.inner = Arc::new(MeshInner {
            vertices: self.inner.vertices.clone(),
            faces: self.inner.faces.clone(),
            normals: normals.into_boxed_slice(),
        });
    }

    /// Intersect the mesh with a plane with linier time complexity. You
    /// should probably use the [`crate::segments::Segments`] struct as it can
    /// massively accelerate slicing of high face count triangles.
    pub fn intersect_plane(&self, height: f32) -> Vec<Pos> {
        // Point is the position of the plane and normal is the direction /
        // rotation of the plane.
        let point = self.inv_transform(&Vector3::new(0.0, 0.0, height));
        let normal = (self.inv_transformation_matrix * Vector3::z_axis().to_homogeneous()).xyz();

        let mut out = Vec::new();

        let vertices = self.vertices();
        for face in self.faces() {
            // Get the vertices of the face
            let v0 = vertices[face[0] as usize];
            let v1 = vertices[face[1] as usize];
            let v2 = vertices[face[2] as usize];

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

    /// Transforms a normal according to the models scale and rotation.
    pub fn transform_normal(&self, normal: &Pos) -> Pos {
        (self.transformation_matrix * normal.to_homogeneous()).xyz()
    }

    /// Undoes the transformation of a point from the models translation, scale, and rotation.
    pub fn inv_transform(&self, pos: &Pos) -> Pos {
        (self.inv_transformation_matrix * pos.push(1.0)).xyz()
    }

    /// Get the minimum and maximum of each component of every vertex in the
    /// model. These points define the bounding box of the model.
    pub fn bounds(&self) -> (Pos, Pos) {
        vertex_bounds(self.vertices(), &self.transformation_matrix)
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

    /// Gets the current rotation of the model.
    pub fn rotation(&self) -> Pos {
        self.rotation
    }
}

/// Loads a buffer into a mesh.
/// Supported formats include:
/// - stl
/// - obj / mtl
pub fn load_mesh<T: BufRead + Seek>(reader: &mut T, format: &str) -> Result<Mesh> {
    let format = format.to_ascii_lowercase();
    Ok(match format.as_str() {
        "stl" => {
            let model = stl_io::read_stl(reader)?;
            let vertices = model
                .vertices
                .iter()
                .map(|v| Pos::new(v[0], v[1], v[2]))
                .collect();
            let faces = model
                .faces
                .iter()
                .map(|f| {
                    [
                        f.vertices[0] as u32,
                        f.vertices[1] as u32,
                        f.vertices[2] as u32,
                    ]
                })
                .collect();
            let normals = model
                .faces
                .iter()
                .map(|f| Vector3::new(f.normal[0], f.normal[1], f.normal[2]))
                .collect();
            Mesh::new(vertices, faces, normals)
        }
        "obj" | "mtl" => {
            let model: Obj<Vertex> = obj::load_obj(reader)?;
            let vertices = model
                .vertices
                .iter()
                .map(|v| Pos::new(v.position[0], v.position[1], v.position[2]))
                .collect();
            let faces = model
                .indices
                .chunks_exact(3)
                .map(|i| [i[0] as u32, i[1] as u32, i[2] as u32])
                .collect::<Vec<_>>();
            let normals = faces
                .iter()
                .map(|face| {
                    let n1 = model.vertices[face[0] as usize].normal;
                    let n2 = model.vertices[face[1] as usize].normal;
                    let n3 = model.vertices[face[2] as usize].normal;
                    let normal = Vector3::new(n1[0], n1[1], n1[2])
                        + Vector3::new(n2[0], n2[1], n2[2])
                        + Vector3::new(n3[0], n3[1], n3[2]);
                    normal.normalize()
                })
                .collect();
            Mesh::new(vertices, faces, normals)
        }
        _ => return Err(anyhow::anyhow!("Unsupported format: {}", format)),
    })
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            inner: Arc::new(MeshInner {
                vertices: Box::new([]),
                faces: Box::new([]),
                normals: Box::new([]),
            }),

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
fn vertex_bounds(vertices: &[Pos], transform: &Matrix4<f32>) -> (Pos, Pos) {
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
    let (min, max) = vertex_bounds(vertices, &Matrix4::identity());

    let center = (min + max) / 2.0;
    let center = Pos::new(center.x, center.y, min.z);
    vertices.iter_mut().for_each(|v| *v -= center);
}

fn recompute_normals(vertices: &[Pos], faces: &[[u32; 3]]) -> Vec<Vector3<f32>> {
    faces
        .iter()
        .map(|f| {
            let edge1 = vertices[f[2] as usize] - vertices[f[1] as usize];
            let edge2 = vertices[f[0] as usize] - vertices[f[1] as usize];
            edge1.cross(&edge2).normalize()
        })
        .collect()
}
