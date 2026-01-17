use egui::PaintCallbackInfo;
use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};
use nalgebra::Vector2;
use wgpu::{CommandBuffer, CommandEncoder, Device, Queue, RenderPass};

mod pipeline;
pub use pipeline::SlicePreviewPipeline;

pub struct SlicePreviewRenderResources {
    pub slice_preview_pipeline: SlicePreviewPipeline,
}

pub struct SlicePreviewRenderCallback {
    pub dimensions: Vector2<u32>,
    pub offset: Vector2<f32>,
    pub aspect: f32,
    pub scale: f32,

    pub new_preview: Option<Vec<u8>>,
}

impl CallbackTrait for SlicePreviewRenderCallback {
    fn prepare(
        &self,
        device: &Device,
        queue: &Queue,
        _screen_descriptor: &ScreenDescriptor,
        _encoder: &mut CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<CommandBuffer> {
        let resources = resources.get_mut::<SlicePreviewRenderResources>().unwrap();

        resources
            .slice_preview_pipeline
            .prepare(device, queue, self);

        Vec::new()
    }

    fn paint(
        &self,
        _info: PaintCallbackInfo,
        render_pass: &mut RenderPass<'static>,
        callback_resources: &CallbackResources,
    ) {
        let resources = callback_resources
            .get::<SlicePreviewRenderResources>()
            .unwrap();
        resources.slice_preview_pipeline.paint(render_pass);
    }
}
