use std::{
    fs::File,
    io::{Read, Write},
    iter,
    path::{Path, PathBuf},
};

use anyhow::Result;
use egui::Color32;
use itertools::Itertools;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{
    app::App,
    render::rendered_mesh::RenderedMesh,
    ui::popup::{Popup, PopupIcon},
};
use common::config::SliceConfig;
use slicer::mesh::Mesh;

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

impl App {
    fn add_recent_project(&mut self, path: PathBuf) {
        self.config.recent_projects = iter::once(path)
            .chain(self.config.recent_projects.iter().cloned())
            .unique()
            .take(5)
            .collect()
    }

    fn _save_project(&mut self, path: &Path) -> Result<()> {
        let meshes = self.meshes.read();
        let project = BorrowedProject::new(&meshes, &self.slice_config);

        let mut file = File::create(path)?;
        project.serialize(&mut file)?;

        drop(meshes);
        self.add_recent_project(path.to_path_buf());
        Ok(())
    }

    pub fn save_project(&mut self, path: &Path) {
        if let Err(error) = self._save_project(path) {
            error!("Error saving project: {:?}", error);
            self.popup.open(Popup::simple(
                "Error Saving Project",
                PopupIcon::Error,
                error.to_string(),
            ));
        }
    }

    fn _load_project(&mut self, path: &Path) -> Result<()> {
        let mut file = File::open(path)?;
        let project = OwnedProject::deserialize(&mut file)?;

        self.add_recent_project(path.to_path_buf());
        project.apply(self);

        info!("Loaded project from `{}`", path.display());
        let meshes = self.meshes.read();
        for (i, mesh) in meshes.iter().enumerate() {
            info!(
                " {} Loaded model `{}` with {} faces",
                if i + 1 < meshes.len() { "│" } else { "└" },
                mesh.name,
                mesh.mesh.face_count()
            );
        }

        Ok(())
    }

    pub fn load_project(&mut self, path: &Path) {
        if let Err(error) = self._load_project(path) {
            error!("Error loading project: {:?}", error);
            self.popup.open(Popup::simple(
                "Error Loading Project",
                PopupIcon::Error,
                error.to_string(),
            ));
        }
    }
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
