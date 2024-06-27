use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device,
};

use slicer::mesh::Mesh;

use super::ModelVertex;

pub struct RenderedMesh {
    pub name: String,
    pub mesh: Mesh,
    pub hidden: bool,
    pub face_count: u32,

    vertices: Vec<ModelVertex>,
    buffers: Option<RenderedMeshBuffers>,
}

pub struct RenderedMeshBuffers {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
}

impl RenderedMesh {
    pub fn from_mesh(mesh: Mesh) -> Self {
        let mut vertices = Vec::new();

        for face in &mesh.faces {
            let (a, b, c) = (
                mesh.vertices[face[0] as usize],
                mesh.vertices[face[1] as usize],
                mesh.vertices[face[2] as usize],
            );
            let normal = (b - a).cross(&(c - a)).normalize();

            vertices.extend_from_slice(&[
                ModelVertex {
                    position: [a.x, a.y, a.z, 1.0],
                    tex_coords: [0.0, 0.0],
                    normal: normal.into(),
                },
                ModelVertex {
                    position: [b.x, b.y, b.z, 1.0],
                    tex_coords: [0.0, 0.0],
                    normal: normal.into(),
                },
                ModelVertex {
                    position: [c.x, c.y, c.z, 1.0],
                    tex_coords: [0.0, 0.0],
                    normal: normal.into(),
                },
            ]);
        }

        Self {
            face_count: mesh.faces.len() as u32,
            mesh,
            name: String::new(),
            hidden: false,
            vertices,
            buffers: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn try_get_buffers(&self) -> Option<&RenderedMeshBuffers> {
        self.buffers.as_ref()
    }

    pub fn get_buffers(&mut self, device: &Device) -> &RenderedMeshBuffers {
        if self.buffers.is_none() {
            let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&self.vertices),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });

            let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(
                    &(0..self.mesh.faces.len() as u32 * 3).collect::<Vec<_>>(),
                ),
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            });

            self.buffers = Some(RenderedMeshBuffers {
                vertex_buffer,
                index_buffer,
            });
        }

        self.buffers.as_ref().unwrap()
    }
}

impl Clone for RenderedMesh {
    fn clone(&self) -> Self {
        Self {
            face_count: self.face_count,
            name: self.name.clone(),
            mesh: self.mesh.clone(),
            hidden: self.hidden,
            vertices: self.vertices.clone(),
            buffers: None,
        }
    }
}
