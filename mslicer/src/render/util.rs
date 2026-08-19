use std::ops::Deref;

use bytemuck::NoUninit;
use slicer::mesh::Mesh;
use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::render::{Gcx, ModelVertex};

#[macro_export]
macro_rules! include_shader {
    ($($shader:literal),*) => {
        wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(concat!(
                $(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/render/shaders/", $shader))),*
            ).into()),
        }
    };
}

pub struct ResizingBuffer {
    inner: Buffer,
}

impl ResizingBuffer {
    pub fn new(device: &Device, usage: BufferUsages) -> Self {
        Self {
            inner: device.create_buffer(&BufferDescriptor {
                label: None,
                size: 0,
                usage: usage | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        }
    }

    pub fn new_sized(device: &Device, usage: BufferUsages, size: u64) -> Self {
        Self {
            inner: device.create_buffer(&BufferDescriptor {
                label: None,
                size,
                usage: usage | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        }
    }

    pub fn write(&mut self, gcx: &Gcx, data: &[u8]) {
        self.resize(gcx, data.len() as u64);
        gcx.queue.write_buffer(&self.inner, 0, data);
    }

    pub fn write_slice<A: NoUninit>(&mut self, gcx: &Gcx, data: &[A]) {
        self.write(gcx, bytemuck::cast_slice(data));
    }

    pub fn resize(&mut self, gcx: &Gcx, size: u64) {
        if size > self.inner.size() {
            self.inner = gcx.device.create_buffer(&BufferDescriptor {
                label: None,
                size: size.next_power_of_two(),
                usage: self.inner.usage(),
                mapped_at_creation: false,
            });
        }
    }
}

impl Deref for ResizingBuffer {
    type Target = Buffer;

    fn deref(&self) -> &Self::Target {
        &self.inner
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
