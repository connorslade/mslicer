use std::sync::{Arc, RwLock};

use egui::PaintCallbackInfo;
use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};
use nalgebra::{Matrix4, Vector3};
use wgpu::{CommandBuffer, CommandEncoder, Device, Queue, RenderPass};

use super::{
    pipelines::{
        build_plate::BuildPlatePipeline,
        model::{ModelPipeline, RenderStyle},
        target_point::TargetPointPipeline,
        Pipeline,
    },
    rendered_mesh::RenderedMesh,
};

pub struct WorkspaceRenderResources {
    pub build_plate_pipeline: BuildPlatePipeline,
    pub model_pipeline: ModelPipeline,
    pub target_point_pipeline: TargetPointPipeline,
}

pub struct WorkspaceRenderCallback {
    pub transform: Matrix4<f32>,

    pub bed_size: Vector3<f32>,
    pub grid_size: f32,

    pub models: Arc<RwLock<Vec<RenderedMesh>>>,
    pub render_style: RenderStyle,

    pub target_point: Vector3<f32>,
    pub is_moving: bool,
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
        let resources = resources.get_mut::<WorkspaceRenderResources>().unwrap();

        resources
            .build_plate_pipeline
            .prepare(device, queue, screen_descriptor, encoder, self);
        resources
            .model_pipeline
            .prepare(device, queue, screen_descriptor, encoder, self);
        resources
            .target_point_pipeline
            .prepare(device, queue, screen_descriptor, encoder, self);

        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: PaintCallbackInfo,
        render_pass: &mut RenderPass<'a>,
        callback_resources: &'a CallbackResources,
    ) {
        let resources = callback_resources
            .get::<WorkspaceRenderResources>()
            .unwrap();

        resources.build_plate_pipeline.paint(render_pass, self);
        resources.model_pipeline.paint(render_pass, self);
        resources.target_point_pipeline.paint(render_pass, self);
    }
}
