use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    iter,
    path::{Path, PathBuf},
    sync::Arc,
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
use slicer::mesh::{Mesh, MeshInner};

const VERSION: u32 = 2;

#[derive(Serialize, Deserialize)]
pub struct Project {
    meshes: Vec<Arc<MeshInner>>,
    models: Vec<ModelInfo>,

    slice_config: SliceConfig,
    post_processing: PostProcessing,
}

#[derive(Serialize, Deserialize)]
pub struct ModelInfo {
    mesh: u32,

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
        let project = Project::new(
            &self.models,
            self.slice_config.clone(),
            self.post_processing.clone(),
        );

        let mut file = File::create(path)?;
        project.serialize(&mut file)?;

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
        let project = Project::deserialize(&mut file)?;

        self.add_recent_project(path.to_path_buf());
        project.apply(self);

        info!("Loaded project from `{}`", path.display());

        let count = self.models.len();
        for (i, mesh) in self.models.iter().enumerate() {
            info!(
                " {} Loaded model `{}` with {} faces",
                if i + 1 < count { "│" } else { "└" },
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

impl Project {
    pub fn new(
        rendered_meshes: &[Model],
        slice_config: SliceConfig,
        post_processing: PostProcessing,
    ) -> Self {
        let mut map = HashMap::new();
        let (mut meshes, mut models) = (Vec::new(), Vec::new());

        for model in rendered_meshes {
            let id = model.mesh.mesh_id();
            let mesh = match map.get(&id) {
                Some(mesh) => *mesh,
                None => {
                    let mesh = map.len() as u32;
                    meshes.push(model.mesh.inner().clone());
                    map.insert(id, mesh);
                    mesh
                }
            };

            models.push(ModelInfo::from_model(model, mesh));
        }

        Self {
            meshes,
            models,

            slice_config,
            post_processing,
        }
    }

    pub fn serialize<Writer: Write>(&self, writer: &mut Writer) -> Result<()> {
        writer.write_all(&VERSION.to_le_bytes())?;
        bincode::serde::encode_into_std_write(self, writer, bincode::config::standard())?;
        Ok(())
    }

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
        app.models = (self.models.into_iter())
            .map(|x| x.into_model(app, &self.meshes))
            .collect();

        app.slice_config = self.slice_config;
        app.post_processing = self.post_processing;
    }
}

impl ModelInfo {
    pub fn from_model(rendered_mesh: &Model, mesh: u32) -> Self {
        Self {
            mesh,

            name: rendered_mesh.name.clone(),
            color: rendered_mesh.color,
            hidden: rendered_mesh.hidden,

            position: rendered_mesh.mesh.position(),
            scale: rendered_mesh.mesh.scale(),
            rotation: rendered_mesh.mesh.rotation(),
        }
    }

    pub fn into_model(self, app: &App, meshes: &[Arc<MeshInner>]) -> Model {
        let mut mesh = Mesh::from_inner(meshes[self.mesh as usize].to_owned());
        mesh.set_position_unchecked(self.position);
        mesh.set_scale_unchecked(self.scale);
        mesh.set_rotation_unchecked(self.rotation);
        mesh.update_transformation_matrix();

        let mut rendered = Model::from_mesh(mesh)
            .with_name(self.name)
            .with_color(self.color)
            .with_hidden(self.hidden);
        rendered.update_oob(&app.slice_config.platform_size);
        rendered
    }
}
