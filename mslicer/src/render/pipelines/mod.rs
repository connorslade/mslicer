use std::ops::Deref;

use bytemuck::NoUninit;
use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device};

use crate::render::Gcx;

mod consts;
pub mod line;
pub mod model;
pub mod point;
pub mod slice_preview;
pub mod support;

#[macro_export]
macro_rules! include_shader {
    ($($shader:literal),*) => {
        wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(concat!($(include_str!(concat!("../shaders/", $shader))),*).into()),
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

    pub fn write(&mut self, gcx: &Gcx, data: &[u8]) {
        if data.len() as u64 > self.inner.size() {
            self.inner = gcx.device.create_buffer(&BufferDescriptor {
                label: None,
                size: (data.len() as u64).next_power_of_two(),
                usage: self.inner.usage(),
                mapped_at_creation: false,
            });
        }

        gcx.queue.write_buffer(&self.inner, 0, data);
    }

    pub fn write_slice<A: NoUninit>(&mut self, gcx: &Gcx, data: &[A]) {
        self.write(gcx, bytemuck::cast_slice(data));
    }
}

impl Deref for ResizingBuffer {
    type Target = Buffer;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
