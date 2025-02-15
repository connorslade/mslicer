use std::sync::Arc;

use egui::PaintCallbackInfo;
use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};
use nalgebra::{Matrix4, Vector3};
use parking_lot::RwLock;
use wgpu::{CommandBuffer, CommandEncoder, Device, Queue, RenderPass};

use crate::app::config::Config;

use super::{
    camera::Camera,
    dispatch::solid_line::SolidLineDispatch,
    pipelines::{model::ModelPipeline, target_point::TargetPointPipeline},
    rendered_mesh::RenderedMesh,
};

pub struct WorkspaceRenderResources {
    pub model_pipeline: ModelPipeline,
    pub target_point_pipeline: TargetPointPipeline,

    pub solid_line: SolidLineDispatch,
}

#[derive(Clone)]
pub struct WorkspaceRenderCallback {
    pub camera: Camera,
    pub transform: Matrix4<f32>,
    pub is_moving: bool,

    pub bed_size: Vector3<f32>,
    pub grid_size: f32,

    pub models: Arc<RwLock<Vec<RenderedMesh>>>,
    pub config: Config,

    pub line_support_debug: Vec<[Vector3<f32>; 2]>,
}

impl CallbackTrait for WorkspaceRenderCallback {
    fn prepare(
        &self,
        device: &Device,
        queue: &Queue,
        _screen_descriptor: &ScreenDescriptor,
        _encoder: &mut CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<CommandBuffer> {
        let resources = resources.get_mut::<WorkspaceRenderResources>().unwrap();

        resources.solid_line.prepare(device, queue, self);
        resources.model_pipeline.prepare(device, self);
        resources.target_point_pipeline.prepare(queue, self);

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

        resources.solid_line.paint(render_pass);
        resources.model_pipeline.paint(render_pass, self);
        resources.target_point_pipeline.paint(render_pass, self);
    }
}
