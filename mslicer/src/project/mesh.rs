use egui::Color32;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

use crate::render::rendered_mesh::RenderedMesh;
use common::serde_impls::{index_list, vector3_list, vector3f};
use slicer::mesh::Mesh;

#[derive(Deserialize)]
pub struct OwnedProjectMesh {
    info: ProjectMeshInfo,

    #[serde(with = "vector3_list")]
    vertices: Vec<Vector3<f32>>,
    #[serde(with = "index_list")]
    faces: Vec<[u32; 3]>,
    #[serde(with = "vector3_list")]
    normals: Vec<Vector3<f32>>,
}

#[derive(Serialize)]
pub struct BorrowedProjectMesh<'a> {
    info: ProjectMeshInfo,

    #[serde(with = "vector3_list")]
    vertices: &'a [Vector3<f32>],
    #[serde(with = "index_list")]
    faces: &'a [[u32; 3]],
    #[serde(with = "vector3_list")]
    normals: &'a [Vector3<f32>],
}

#[derive(Serialize, Deserialize)]
pub struct ProjectMeshInfo {
    name: String,
    #[serde(with = "color32")]
    color: Color32,
    hidden: bool,

    #[serde(with = "vector3f")]
    position: Vector3<f32>,
    #[serde(with = "vector3f")]
    scale: Vector3<f32>,
    #[serde(with = "vector3f")]
    rotation: Vector3<f32>,
}

impl OwnedProjectMesh {
    pub fn into_rendered_mesh(self) -> RenderedMesh {
        let mut mesh = Mesh::new_uncentred(self.vertices, self.faces, self.normals);
        mesh.set_position_unchecked(self.info.position);
        mesh.set_scale_unchecked(self.info.scale);
        mesh.set_rotation_unchecked(self.info.rotation);
        mesh.update_transformation_matrix();

        RenderedMesh::from_mesh(mesh)
            .with_name(self.info.name)
            .with_color(self.info.color)
            .with_hidden(self.info.hidden)
    }
}

impl<'a> BorrowedProjectMesh<'a> {
    pub fn from_rendered_mesh(rendered_mesh: &'a RenderedMesh) -> Self {
        Self {
            info: ProjectMeshInfo::from_rendered_mesh(rendered_mesh),

            vertices: rendered_mesh.mesh.vertices(),
            faces: rendered_mesh.mesh.faces(),
            normals: rendered_mesh.mesh.normals(),
        }
    }
}

impl ProjectMeshInfo {
    pub fn from_rendered_mesh(rendered_mesh: &RenderedMesh) -> Self {
        Self {
            name: rendered_mesh.name.clone(),
            color: rendered_mesh.color,
            hidden: rendered_mesh.hidden,

            position: rendered_mesh.mesh.position(),
            scale: rendered_mesh.mesh.scale(),
            rotation: rendered_mesh.mesh.rotation(),
        }
    }
}

pub mod color32 {
    use egui::Color32;

    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color32, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let [r, g, b, a]: [u8; 4] = <[u8; 4]>::deserialize(deserializer)?;
        Ok(Color32::from_rgba_premultiplied(r, g, b, a))
    }

    pub fn serialize<S>(data: &Color32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        data.to_array().serialize(serializer)
    }
}
