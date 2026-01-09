use std::sync::Arc;

use egui::PaintCallbackInfo;
use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};
use nalgebra::{Matrix4, Vector3};
use parking_lot::RwLock;
use wgpu::{CommandBuffer, CommandEncoder, Device, Queue, RenderPass};

use crate::{app::config::Config, render::dispatch::point::PointDispatch};

use super::{
    camera::Camera, dispatch::line::LineDispatch, model::Model, pipelines::model::ModelPipeline,
};

pub struct WorkspaceRenderResources {
    pub model_pipeline: ModelPipeline,

    pub point: PointDispatch,
    pub solid_line: LineDispatch,
}

#[derive(Clone)]
pub struct WorkspaceRenderCallback {
    pub camera: Camera,
    pub transform: Matrix4<f32>,
    pub is_moving: bool,

    pub bed_size: Vector3<f32>,
    pub grid_size: f32,

    pub models: Arc<RwLock<Vec<Model>>>,
    pub config: Config,

    pub line_support_debug: Vec<[Vector3<f32>; 2]>,
    pub overhang_angle: f32,
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

        resources.model_pipeline.prepare(device, self);

        resources.solid_line.prepare(device, queue, self);
        resources.point.prepare(device, queue, self);

        Vec::new()
    }

    fn paint(
        &self,
        _info: PaintCallbackInfo,
        render_pass: &mut RenderPass,
        callback_resources: &CallbackResources,
    ) {
        let resources = callback_resources
            .get::<WorkspaceRenderResources>()
            .unwrap();

        resources.solid_line.paint(render_pass);
        resources.model_pipeline.paint(render_pass, self);
        resources.point.paint(render_pass);
    }
}
