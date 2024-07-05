use egui_wgpu::ScreenDescriptor;
use wgpu::{CommandEncoder, Device, Queue, RenderPass};

pub mod build_plate;
mod consts;
pub mod model;
pub mod slice_preview;
pub mod target_point;

pub trait Pipeline<T> {
    fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        screen_descriptor: &ScreenDescriptor,
        encoder: &mut CommandEncoder,
        resources: &T,
    );
    fn paint<'a>(&'a self, render_pass: &mut RenderPass<'a>, resources: &T);
}

#[macro_export]
macro_rules! include_shader {
    ($shader:literal) => {
        ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(include_str!(concat!("../../shaders/", $shader)).into()),
        }
    };
}
