use std::sync::Arc;

use egui::PaintCallbackInfo;
use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};
use nalgebra::{Matrix4, Vector3};
use parking_lot::RwLock;
use wgpu::{CommandBuffer, CommandEncoder, Device, Queue, RenderPass};

use crate::{
    app::config::Config,
    render::{dispatch::point::PointDispatch, pipelines::support::SupportPipeline},
};

use super::{
    camera::Camera, dispatch::line::LineDispatch, model::Model, pipelines::model::ModelPipeline,
};

pub struct WorkspaceRenderResources {
    pub model: ModelPipeline,
    pub support: SupportPipeline,

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
    pub overhang_angle: Option<f32>,
}

pub struct Gcx<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
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

        let gcx = Gcx { device, queue };
        resources.model.prepare(&gcx, self);
        resources.support.prepare(&gcx, self);
        resources.solid_line.prepare(&gcx, self);
        resources.point.prepare(&gcx, self);

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
        resources.model.paint(render_pass, self);
        resources.point.paint(render_pass);
        resources.support.paint(render_pass);
    }
}
