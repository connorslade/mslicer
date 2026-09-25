use std::{mem, ops::Deref};

use bytemuck::NoUninit;
use nalgebra::Vector3;
use slicer::mesh::Mesh;
use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::render::Gcx;

#[macro_export]
macro_rules! include_shader {
    ($($shader:literal),*) => {
        wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(concat!(
                $(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/", $shader))),*
            ).into()),
        }
    };
}

pub struct ResizingBuffer {
    inner: Buffer,
}

pub struct MeshBuffers {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
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

    pub fn write(&mut self, gcx: &Gcx, data: &[u8]) -> bool {
        let resized = self.resize(gcx, data.len() as u64);
        gcx.queue.write_buffer(&self.inner, 0, data);
        resized
    }

    pub fn write_slice<A: NoUninit>(&mut self, gcx: &Gcx, data: &[A]) -> bool {
        self.write(gcx, bytemuck::cast_slice(data))
    }

    pub fn resize(&mut self, gcx: &Gcx, size: u64) -> bool {
        if size > self.inner.size() {
            self.inner = gcx.device.create_buffer(&BufferDescriptor {
                label: None,
                size: size.next_power_of_two(),
                usage: self.inner.usage(),
                mapped_at_creation: false,
            });
            return true;
        }

        false
    }
}

impl MeshBuffers {
    pub fn get_for(device: &Device, mesh: &Mesh) -> Self {
        let (vertex_buffer, index_buffer) = gpu_mesh_buffers(device, mesh);
        Self {
            vertex_buffer,
            index_buffer,
        }
    }
}

impl Deref for ResizingBuffer {
    type Target = Buffer;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub fn gpu_mesh(mesh: &Mesh) -> (&[u8], &[u8]) {
    let vertices = unsafe { mem::transmute::<&[Vector3<f32>], &[[f32; 3]]>(mesh.vertices()) };
    (
        bytemuck::cast_slice(vertices),
        bytemuck::cast_slice(mesh.faces()),
    )
}

pub fn gpu_mesh_buffers(device: &Device, mesh: &Mesh) -> (Buffer, Buffer) {
    let (vertex, index) = gpu_mesh(mesh);
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(vertex),
        usage: BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: index,
        usage: BufferUsages::INDEX,
    });

    (vertex_buffer, index_buffer)
}
