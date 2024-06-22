use std::{
    mem,
    sync::{Arc, RwLock},
};

use egui_wgpu::CallbackTrait;
use nalgebra::{Matrix4, Vector3};
use pipelines::{build_plate::BuildPlatePipeline, model::ModelPipeline, CachedPipeline, Pipeline};
use wgpu::{BindGroup, Buffer, COPY_BUFFER_ALIGNMENT};

use rendered_mesh::RenderedMesh;
pub mod camera;
pub mod pipelines;
pub mod render;
pub mod rendered_mesh;

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum RenderStyle {
    Normals,
    Rended,
}

pub struct WorkspaceRenderResources {
    pub uniform_buffer: Buffer,
    pub bind_group: BindGroup,

    build_plate_pipeline: CachedPipeline<BuildPlatePipeline>,
    model_pipeline: CachedPipeline<ModelPipeline>,
}

pub struct WorkspaceRenderCallback {
    pub bed_size: Vector3<f32>,
    pub transform: Matrix4<f32>,
    pub models: Arc<RwLock<Vec<RenderedMesh>>>,
    pub render_style: RenderStyle,
}

impl CallbackTrait for WorkspaceRenderCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources = callback_resources
            .get::<WorkspaceRenderResources>()
            .unwrap();

        // todo: bring into Pipeline::prepare
        let mut to_generate = Vec::new();
        for (idx, model) in self.models.read().unwrap().iter().enumerate() {
            if model.try_get_buffers().is_none() {
                to_generate.push(idx);
            }
        }

        if !to_generate.is_empty() {
            let mut meshes = self.models.write().unwrap();
            for idx in to_generate {
                meshes[idx].get_buffers(device);
            }
        }

        // todo: only do on change
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

        render_pass.set_bind_group(0, &resources.bind_group, &[]);

        render_pass.set_pipeline(resources.build_plate_pipeline.get_render_pipeline());
        resources
            .build_plate_pipeline
            .pipeline
            .paint(render_pass, self);

        render_pass.set_pipeline(resources.model_pipeline.get_render_pipeline());
        resources.model_pipeline.pipeline.paint(render_pass, self);
    }
}

impl WorkspaceRenderCallback {
    const PADDED_SIZE: u64 = ((mem::size_of::<Self>() as u64 + COPY_BUFFER_ALIGNMENT - 1)
        / COPY_BUFFER_ALIGNMENT)
        * COPY_BUFFER_ALIGNMENT;

    fn to_wgsl(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::PADDED_SIZE as usize);
        out.extend_from_slice(bytemuck::cast_slice(self.bed_size.as_slice()));
        out.extend_from_slice(&[0, 0, 0, 0]);
        out.extend_from_slice(bytemuck::cast_slice(self.transform.as_slice()));
        out.push(self.render_style as u8);
        out.resize(Self::PADDED_SIZE as usize, 0);
        out
    }
}

impl RenderStyle {
    pub fn name(&self) -> &'static str {
        match self {
            RenderStyle::Normals => "Normals",
            RenderStyle::Rended => "Rended",
        }
    }
}
