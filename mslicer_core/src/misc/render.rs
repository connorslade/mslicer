use nalgebra::Vector4;
use slicer::mesh::Mesh;
use wgpu::{
    Buffer, BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 4],
}

impl ModelVertex {
    pub fn new(pos: Vector4<f32>) -> Self {
        Self {
            position: [pos.x, pos.y, pos.z, pos.w],
        }
    }
}

pub fn gpu_mesh(mesh: &Mesh) -> (Vec<ModelVertex>, Vec<u32>) {
    let index = mesh.faces().iter().flatten().copied().collect::<Vec<_>>();
    let vertices = (mesh.vertices().iter())
        .map(|vert| ModelVertex::new(vert.push(1.0)))
        .collect::<Vec<_>>();
    (vertices, index)
}

pub fn gpu_mesh_buffers(device: &Device, mesh: &Mesh) -> (Buffer, Buffer) {
    let (vertices, indices) = gpu_mesh(mesh);

    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&vertices),
        usage: BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&indices),
        usage: BufferUsages::INDEX,
    });

    (vertex_buffer, index_buffer)
}
