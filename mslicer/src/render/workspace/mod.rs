use std::sync::Arc;

use egui::PaintCallbackInfo;
use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};
use nalgebra::{Matrix4, Vector3};
use parking_lot::RwLock;
use slicer::mesh::Mesh;
use wgpu::{CommandBuffer, CommandEncoder, Device, Queue, RenderPass};

use crate::{
    app::{config::Config, model::Model},
    render::{
        Gcx,
        camera::Camera,
        workspace::{
            line::LineDispatch, model::ModelPipeline, point::PointDispatch,
            support::SupportPipeline,
        },
    },
};

pub mod line;
pub mod model;
pub mod point;
pub mod support;

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
    pub support_model: Option<Mesh>,
    pub overhang_angle: Option<f32>,
}

impl CallbackTrait for WorkspaceRenderCallback {
    fn prepare(
        &self,
        device: &Device,
        queue: &Queue,
        _screen: &ScreenDescriptor,
        _encoder: &mut CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<CommandBuffer> {
        let workspace = resources.get_mut::<WorkspaceRenderResources>().unwrap();
        let gcx = Gcx { device, queue };
        workspace.model.prepare(&gcx, self);
        workspace.support.prepare(&gcx, self);
        workspace.solid_line.prepare(&gcx, self);
        workspace.point.prepare(&gcx, self);

        Vec::new()
    }

    fn paint(
        &self,
        _info: PaintCallbackInfo,
        render_pass: &mut RenderPass,
        resources: &CallbackResources,
    ) {
        let workspace = resources.get::<WorkspaceRenderResources>().unwrap();
        workspace.solid_line.paint(render_pass);
        workspace.model.paint(render_pass, self);
        workspace.point.paint(render_pass);
        workspace.support.paint(render_pass);
    }
}
