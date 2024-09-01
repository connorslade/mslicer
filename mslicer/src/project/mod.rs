use std::io::{Read, Write};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{app::App, render::rendered_mesh::RenderedMesh};
use common::config::SliceConfig;

use mesh::{BorrowedProjectMesh, OwnedProjectMesh};
mod mesh;

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
