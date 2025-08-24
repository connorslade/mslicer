use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};
use nalgebra::Vector2;
use wgpu::{CommandBuffer, CommandEncoder, Device, Queue};

use super::pipelines::slice_preview::SlicePreviewPipeline;

pub struct SlicePreviewRenderResources {
    pub slice_preview_pipeline: SlicePreviewPipeline,
}

pub struct SlicePreviewRenderCallback {
    pub dimensions: Vector2<u32>,
    pub offset: Vector2<f32>,
    pub aspect: f32,
    pub scale: f32,

    pub new_preview: Option<Vec<u8>>,
    pub new_annotations: Option<Vec<u8>>,
    pub show_hide: u32,
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

    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut egui_wgpu::wgpu::RenderPass<'a>,
        callback_resources: &'a egui_wgpu::CallbackResources,
    ) {
        let resources = callback_resources
            .get::<SlicePreviewRenderResources>()
            .unwrap();

        resources.slice_preview_pipeline.paint(render_pass);
    }
}
