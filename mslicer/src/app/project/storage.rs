use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::{Result, ensure};
use nalgebra::Vector3;

use crate::app::project::{PostProcessing, Project, model::Model};
use common::{
    progress::Progress,
    serde::{Deserializer, SerdeExt, Serializer},
    slice::SliceConfig,
};
use slicer::{
    mesh::{Mesh, MeshInner},
    post_process::{anti_alias::AntiAlias, elephant_foot_fixer::ElephantFootFixer},
};

const VERSION: u16 = 2;

struct ModelInfo {
    mesh: u32,

    name: String,
    color: Vector3<f32>,
    hidden: bool,

    position: Vector3<f32>,
    scale: Vector3<f32>,
    rotation: Vector3<f32>,

    file_path: Option<PathBuf>,
    parent_model_id: Option<u32>,
}

impl ModelInfo {
    pub fn new(mesh: u32, model: &Model) -> Self {
        Self {
            mesh,
            name: model.name.to_owned(),
            color: model.color.into(),
            hidden: model.hidden,
            position: model.mesh.position(),
            scale: model.mesh.scale(),
            rotation: model.mesh.rotation(),
            file_path: model.file_path.clone(),
            parent_model_id: model.parent_model_id,
        }
    }

    pub fn into_model(self, inner: Arc<MeshInner>) -> Model {
        let mut mesh = Mesh::from_inner(inner);
        mesh.set_position_unchecked(self.position);
        mesh.set_scale_unchecked(self.scale);
        mesh.set_rotation_unchecked(self.rotation);
        mesh.update_transformation_matrix();

        let mut model = Model::from_mesh(mesh)
            .with_name(self.name)
            .with_color(self.color.into())
            .with_hidden(self.hidden);
        if let Some(file_path) = self.file_path {
            model = model.with_file_path(file_path);
        }
        model
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        // Mesh reference
        ser.write_u32_be(self.mesh);

        // Model properties
        ser.write_u32_be(self.name.len() as u32);
        ser.write_bytes(self.name.as_bytes());
        Vector3::from(self.color).serialize(ser);
        ser.write_bool(self.hidden);

        // Mesh properties
        self.position.serialize(ser);
        self.scale.serialize(ser);
        self.rotation.serialize(ser);

        // File path
        ser.write_bool(self.file_path.is_some());
        if let Some(file_path) = &self.file_path {
            let path_str = file_path.to_string_lossy();
            ser.write_u32_be(path_str.len() as u32);
            ser.write_bytes(path_str.as_bytes());
        }

        // Parent model ID
        ser.write_bool(self.parent_model_id.is_some());
        if let Some(parent_id) = self.parent_model_id {
            ser.write_u32_be(parent_id);
        }
    }

    pub fn deserialize<T: Deserializer>(des: &mut T) -> Self {
        Self {
            mesh: des.read_u32_be(),
            name: {
                let name_len = des.read_u32_be();
                let data = des.read_bytes(name_len as usize);
                String::from_utf8_lossy(&data).into_owned()
            },
            color: Vector3::<f32>::deserialize(des),
            hidden: des.read_bool(),
            position: Vector3::<f32>::deserialize(des),
            scale: Vector3::<f32>::deserialize(des),
            rotation: Vector3::<f32>::deserialize(des),
            file_path: {
                let has_path = des.read_bool();
                if has_path {
                    let path_len = des.read_u32_be();
                    let path_data = des.read_bytes(path_len as usize);
                    let path_str = String::from_utf8_lossy(&path_data);
                    Some(PathBuf::from(&*path_str))
                } else {
                    None
                }
            },
            parent_model_id: {
                let has_parent = des.read_bool();
                if has_parent {
                    Some(des.read_u32_be())
                } else {
                    None
                }
            },
        }
    }
}

impl Project {
    pub fn serialize<T: Serializer>(&self, ser: &mut T, progress: Progress) {
        ser.write_u16_be(VERSION);
        self.slice_config.serialize(ser);
        self.post_processing.serialize(ser);

        let mut total = 0;
        let mut map = HashMap::new();
        let mut meshes = Vec::new();

        ser.write_u32_be(self.models.len() as u32);
        for model in self.models.iter() {
            let id = model.mesh.mesh_id();
            let mesh = match map.get(&id) {
                Some(mesh) => *mesh,
                None => {
                    let mesh = map.len() as u32;
                    meshes.push(model.mesh.inner().clone());
                    total += model.mesh.vertex_count() + model.mesh.face_count();
                    map.insert(id, mesh);
                    mesh
                }
            };

            let info = ModelInfo::new(mesh, model);
            info.serialize(ser);
        }

        progress.set_total(total as u64);
        ser.write_u32_be(meshes.len() as u32);
        (meshes.iter()).for_each(|mesh| serialize_mesh_inner(ser, mesh, &progress));
        progress.set_finished();
    }

    pub fn deserialize<T: Deserializer>(des: &mut T, progress: Progress) -> Result<Self> {
        ensure!(des.read_u16_be() == VERSION, "Save version mismatch.");
        let slice_config = SliceConfig::deserialize(des)?;
        let post_processing = PostProcessing::deserialize(des);

        let models = des.read_u32_be();
        let models = (0..models)
            .map(|_| ModelInfo::deserialize(des))
            .collect::<Vec<_>>();

        let meshes = des.read_u32_be();
        progress.set_total((des.size() - des.pos()) as u64);
        let meshes = (0..meshes)
            .map(|_| Arc::new(deserialize_mesh_inner(des, &progress)))
            .collect::<Vec<_>>();

        let models = (models.into_iter())
            .map(|x| {
                let mesh = meshes[x.mesh as usize].clone();
                x.into_model(mesh)
            })
            .collect();

        progress.set_finished();
        Ok(Self {
            path: None,
            slice_config,
            post_processing,
            models,
        })
    }
}

impl PostProcessing {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        self.anti_alias.serialize(ser);
        self.elephant_foot_fixer.serialize(ser);
    }

    pub fn deserialize<T: Deserializer>(des: &mut T) -> Self {
        Self {
            anti_alias: AntiAlias::deserialize(des),
            elephant_foot_fixer: ElephantFootFixer::deserialize(des),
        }
    }
}

fn serialize_mesh_inner<T: Serializer>(ser: &mut T, mesh: &Arc<MeshInner>, progress: &Progress) {
    ser.write_u32_be(mesh.vertices.len() as u32);
    for vert in mesh.vertices.iter() {
        vert.serialize(ser);
        progress.add_complete(1);
    }

    ser.write_u32_be(mesh.faces.len() as u32);
    for face in mesh.faces.iter() {
        ser.write_u32_be(face[0]);
        ser.write_u32_be(face[1]);
        ser.write_u32_be(face[2]);
        progress.add_complete(3);
    }
}

fn deserialize_mesh_inner<T: Deserializer>(des: &mut T, progress: &Progress) -> MeshInner {
    let verts = des.read_u32_be();
    let verts = (0..verts)
        .map(|_| Vector3::<f32>::deserialize(des))
        .inspect(|_| progress.add_complete(4 * 3))
        .collect::<Vec<_>>();

    let faces = des.read_u32_be();
    let faces = (0..faces)
        .map(|_| [des.read_u32_be(), des.read_u32_be(), des.read_u32_be()])
        .inspect(|_| progress.add_complete(4 * 3))
        .collect::<Vec<_>>();

    MeshInner {
        vertices: verts.into_boxed_slice(),
        faces: faces.into_boxed_slice(),
    }
}
