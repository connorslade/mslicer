use std::ops::Deref;

use bytemuck::NoUninit;
use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device};

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

impl Deref for ResizingBuffer {
    type Target = Buffer;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
