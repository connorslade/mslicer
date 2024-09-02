use std::io::{Read, Write};

use anyhow::Result;
use egui::Color32;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use slicer::mesh::Mesh;

use crate::{app::App, render::rendered_mesh::RenderedMesh};
use common::config::SliceConfig;

const VERSION: u32 = 0;

#[derive(Serialize)]
pub struct BorrowedProject<'a> {
    meshes: Vec<BorrowedProjectMesh<'a>>,
    slice_config: &'a SliceConfig,
}

#[derive(Deserialize)]
pub struct OwnedProject {
    meshes: Vec<OwnedProjectMesh>,
    slice_config: SliceConfig,
}

#[derive(Deserialize)]
pub struct OwnedProjectMesh {
    info: ProjectMeshInfo,

    vertices: Vec<Vector3<f32>>,
    faces: Vec<[u32; 3]>,
    normals: Vec<Vector3<f32>>,
}

#[derive(Serialize)]
pub struct BorrowedProjectMesh<'a> {
    info: ProjectMeshInfo,

    vertices: &'a [Vector3<f32>],
    faces: &'a [[u32; 3]],
    normals: &'a [Vector3<f32>],
}

#[derive(Serialize, Deserialize)]
pub struct ProjectMeshInfo {
    name: String,
    #[serde(with = "color32")]
    color: Color32,
    hidden: bool,

    position: Vector3<f32>,
    scale: Vector3<f32>,
    rotation: Vector3<f32>,
}

impl<'a> BorrowedProject<'a> {
    pub fn new(meshes: &'a [RenderedMesh], slice_config: &'a SliceConfig) -> Self {
        let meshes = meshes
            .iter()
            .map(BorrowedProjectMesh::from_rendered_mesh)
            .collect();

        Self {
            meshes,
            slice_config,
        }
    }

    pub fn serialize<Writer: Write>(&self, writer: &mut Writer) -> Result<()> {
        writer.write_all(&VERSION.to_le_bytes())?;
        bincode::serialize_into(writer, self)?;
        Ok(())
    }
}

impl OwnedProject {
    pub fn deserialize<Reader: Read>(reader: &mut Reader) -> Result<Self> {
        let mut version_bytes = [0; 4];
        reader.read_exact(&mut version_bytes)?;
        let version = u32::from_le_bytes(version_bytes);

        if version != VERSION {
            anyhow::bail!("Invalid version: Expected {VERSION} found {version}");
        }

        Ok(bincode::deserialize_from(reader)?)
    }

    pub fn apply(self, app: &mut App) {
        let mut meshes = app.meshes.write();
        *meshes = self
            .meshes
            .into_iter()
            .map(|mesh| mesh.into_rendered_mesh())
            .collect();

        app.slice_config = self.slice_config;
    }
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
