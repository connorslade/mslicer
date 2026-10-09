use egui::PaintCallbackInfo;
use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};
use nalgebra::Vector2;
use wgpu::{CommandBuffer, CommandEncoder, Device, Queue, RenderPass};

use crate::{
    core::App,
    render::workspace::{
        line::LineDispatch,
        model::{ModelPipeline, Target},
        point::PointDispatch,
        support::SupportPipeline,
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

// todo: do i even have to say it...
pub struct WorkspaceRenderCallback {
    pub app: *mut App,
}

pub struct PreviewRenderCallback {
    pub app: *mut App,
    pub viewport: Vector2<u32>,
}

impl CallbackTrait for WorkspaceRenderCallback {
    fn prepare(
        &self,
        _device: &Device,
        _queue: &Queue,
        screen: &ScreenDescriptor,
        encoder: &mut CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<CommandBuffer> {
        let workspace = resources.get_mut::<WorkspaceRenderResources>().unwrap();
        let app = unsafe { &mut *self.app };
        let gcx = app.gcx();

        workspace.model.prepare(&gcx, screen, encoder, app);
        workspace.support.prepare(&gcx, app);
        workspace.solid_line.prepare(&gcx, app);
        workspace.point.prepare(&gcx, app);

        Vec::new()
    }

    fn paint(
        &self,
        _info: PaintCallbackInfo,
        render_pass: &mut RenderPass,
        resources: &CallbackResources,
    ) {
        let workspace = resources.get::<WorkspaceRenderResources>().unwrap();
        let app = unsafe { &mut *self.app };

        workspace.solid_line.paint(render_pass);
        workspace.model.paint(render_pass, app, Target::Viewport);
        workspace.point.paint(render_pass);
        workspace.support.paint(render_pass, app);
    }
}

// do u really have to put the word 'Trait' in ur trait name...
impl CallbackTrait for PreviewRenderCallback {
    fn prepare(
        &self,
        _device: &Device,
        _queue: &Queue,
        _screen: &ScreenDescriptor,
        encoder: &mut CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let workspace = resources.get_mut::<WorkspaceRenderResources>().unwrap();
        let app = unsafe { &mut *self.app };
        let (gcx, view) = (app.gcx(), self.viewport);

        workspace.model.prepare_preview(&gcx, encoder, app, view);

        Vec::new()
    }

    fn paint(
        &self,
        _info: PaintCallbackInfo,
        render_pass: &mut RenderPass,
        resources: &CallbackResources,
    ) {
        let workspace = resources.get::<WorkspaceRenderResources>().unwrap();
        let app = unsafe { &mut *self.app };

        workspace.model.paint(render_pass, app, Target::Preview);
    }
}

unsafe impl Send for WorkspaceRenderCallback {}
unsafe impl Sync for WorkspaceRenderCallback {}

unsafe impl Send for PreviewRenderCallback {}
unsafe impl Sync for PreviewRenderCallback {}
