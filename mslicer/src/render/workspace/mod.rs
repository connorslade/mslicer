use egui::PaintCallbackInfo;
use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};
use wgpu::{CommandBuffer, CommandEncoder, Device, Queue, RenderPass};

use crate::{
    app::App,
    render::{
        Gcx,
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

pub struct WorkspaceRenderCallback {
    pub app: *mut App,
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
        let app = self.app();

        workspace.model.prepare(&gcx, app);
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
        let app = self.app();

        workspace.solid_line.paint(render_pass);
        workspace.model.paint(render_pass, app);
        workspace.point.paint(render_pass);
        workspace.support.paint(render_pass);
    }
}

impl WorkspaceRenderCallback {
    #[allow(clippy::mut_from_ref)]
    pub fn app(&self) -> &mut App {
        unsafe { &mut *self.app }
    }
}

unsafe impl Send for WorkspaceRenderCallback {}
unsafe impl Sync for WorkspaceRenderCallback {}
