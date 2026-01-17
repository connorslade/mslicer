use std::{
    fs::File,
    io::{Read, Write},
    iter,
    path::{Path, PathBuf},
};

use anyhow::Result;
use itertools::Itertools;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{
    app::{App, PostProcessing, model::Model},
    ui::popup::{Popup, PopupIcon},
};
use common::{color::LinearRgb, config::SliceConfig};
use slicer::mesh::Mesh;

const VERSION: u32 = 1;

#[derive(Serialize)]
pub struct BorrowedProject<'a> {
    meshes: Vec<BorrowedProjectMesh<'a>>,
    slice_config: &'a SliceConfig,
    post_processing: &'a PostProcessing,
}

#[derive(Deserialize)]
pub struct OwnedProject {
    meshes: Vec<OwnedProjectMesh>,
    slice_config: SliceConfig,
    post_processing: PostProcessing,
}

#[derive(Deserialize)]
pub struct OwnedProjectMesh {
    info: ProjectMeshInfo,

    vertices: Vec<Vector3<f32>>,
    faces: Vec<[u32; 3]>,
}

#[derive(Serialize)]
pub struct BorrowedProjectMesh<'a> {
    info: ProjectMeshInfo,

    vertices: &'a [Vector3<f32>],
    faces: &'a [[u32; 3]],
}

#[derive(Serialize, Deserialize)]
pub struct ProjectMeshInfo {
    name: String,
    color: LinearRgb<f32>,
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
        let meshes = self.models.read();
        let project = BorrowedProject::new(&meshes, &self.slice_config, &self.post_processing);

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
        let meshes = self.models.read();
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
    pub fn new(
        meshes: &'a [Model],
        slice_config: &'a SliceConfig,
        post_processing: &'a PostProcessing,
    ) -> Self {
        let meshes = (meshes.iter())
            .map(BorrowedProjectMesh::from_rendered_mesh)
            .collect();

        Self {
            meshes,
            slice_config,
            post_processing,
        }
    }

    pub fn serialize<Writer: Write>(&self, writer: &mut Writer) -> Result<()> {
        writer.write_all(&VERSION.to_le_bytes())?;
        bincode::serde::encode_into_std_write(self, writer, bincode::config::standard())?;
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

        Ok(bincode::serde::decode_from_std_read(
            reader,
            bincode::config::standard(),
        )?)
    }

    pub fn apply(self, app: &mut App) {
        let mut meshes = app.models.write();
        *meshes = self
            .meshes
            .into_iter()
            .map(|mesh| mesh.into_rendered_mesh(app))
            .collect();

        app.slice_config = self.slice_config;
        app.post_processing = self.post_processing;
    }
}

impl OwnedProjectMesh {
    pub fn into_rendered_mesh(self, app: &App) -> Model {
        let mut mesh = Mesh::new_uncentred(self.vertices, self.faces);
        mesh.set_position_unchecked(self.info.position);
        mesh.set_scale_unchecked(self.info.scale);
        mesh.set_rotation_unchecked(self.info.rotation);
        mesh.update_transformation_matrix();

        let mut rendered = Model::from_mesh(mesh)
            .with_name(self.info.name)
            .with_color(self.info.color)
            .with_hidden(self.info.hidden);
        rendered.update_oob(&app.slice_config);
        rendered
    }
}

impl<'a> BorrowedProjectMesh<'a> {
    pub fn from_rendered_mesh(rendered_mesh: &'a Model) -> Self {
        Self {
            info: ProjectMeshInfo::from_rendered_mesh(rendered_mesh),

            vertices: rendered_mesh.mesh.vertices(),
            faces: rendered_mesh.mesh.faces(),
        }
    }
}

impl ProjectMeshInfo {
    pub fn from_rendered_mesh(rendered_mesh: &Model) -> Self {
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
