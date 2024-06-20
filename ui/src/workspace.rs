use egui_wgpu::CallbackTrait;
use nalgebra::Matrix4;
use wgpu::{BindGroup, Buffer, IndexFormat, RenderPipeline};

use crate::render::ModelVertex;
use slicer::mesh::Mesh;

pub struct WorkspaceRenderResources {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub uniform_buffer: Buffer,

    pub render_pipeline: RenderPipeline,
    pub bind_group: BindGroup,

    pub modal: Mesh,
}

pub struct WorkspaceRenderCallback {
    pub transform: Matrix4<f32>,
}

impl CallbackTrait for WorkspaceRenderCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources = callback_resources
            .get::<WorkspaceRenderResources>()
            .unwrap();

        let vertices = resources
            .modal
            .vertices
            .iter()
            .map(|v| ModelVertex {
                position: [v.x, v.y, v.z, 1.0],
                tex_coords: [0.0, 0.0],
            })
            .collect::<Vec<ModelVertex>>();

        queue.write_buffer(&resources.vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        queue.write_buffer(
            &resources.index_buffer,
            0,
            bytemuck::cast_slice(&resources.modal.faces),
        );

        queue.write_buffer(&resources.uniform_buffer, 0, &self.to_wgsl());

        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut egui_wgpu::wgpu::RenderPass<'a>,
        callback_resources: &'a egui_wgpu::CallbackResources,
    ) {
        let resources = callback_resources
            .get::<WorkspaceRenderResources>()
            .unwrap();

        render_pass.set_pipeline(&resources.render_pipeline);
        render_pass.set_bind_group(0, &resources.bind_group, &[]);
        render_pass.set_vertex_buffer(0, resources.vertex_buffer.slice(..));
        render_pass.set_index_buffer(resources.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..(3 * resources.modal.faces.len() as u32), 0, 0..1);
    }
}

impl WorkspaceRenderCallback {
    fn to_wgsl(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(bytemuck::cast_slice(self.transform.as_slice()));
        out
    }
}
