use std::{
    mem,
    sync::{Arc, RwLock},
};

use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};
use nalgebra::{Matrix4, Vector3};
use pipelines::{
    build_plate::BuildPlatePipeline,
    model::{ModelPipeline, RenderStyle},
    Pipeline,
};
use wgpu::{CommandBuffer, CommandEncoder, Device, Queue, COPY_BUFFER_ALIGNMENT};

use rendered_mesh::RenderedMesh;
pub mod camera;
pub mod pipelines;
pub mod render;
pub mod rendered_mesh;

pub struct WorkspaceRenderResources {
    build_plate_pipeline: BuildPlatePipeline,
    model_pipeline: ModelPipeline,
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
        device: &Device,
        queue: &Queue,
        screen_descriptor: &ScreenDescriptor,
        encoder: &mut CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<CommandBuffer> {
        let resources = resources.get::<WorkspaceRenderResources>().unwrap();

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

        resources
            .build_plate_pipeline
            .prepare(device, queue, screen_descriptor, encoder, self);
        resources
            .model_pipeline
            .prepare(device, queue, screen_descriptor, encoder, self);

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

        resources.build_plate_pipeline.paint(render_pass, self);
        resources.model_pipeline.paint(render_pass, self);
    }
}
