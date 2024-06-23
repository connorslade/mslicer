use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};
use wgpu::{CommandBuffer, CommandEncoder, Device, Queue};

use super::pipelines::{slice_preview::SlicePreviewPipeline, Pipeline};

pub struct SlicePreviewRenderResources {
    pub slice_preview_pipeline: SlicePreviewPipeline,
}

pub struct SlicePreviewRenderCallback {
    pub dimensions: (u32, u32),
    pub new_preview: Option<Vec<u8>>,
}

impl CallbackTrait for SlicePreviewRenderCallback {
    fn prepare(
        &self,
        device: &Device,
        queue: &Queue,
        screen_descriptor: &ScreenDescriptor,
        encoder: &mut CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<CommandBuffer> {
        let resources = resources.get_mut::<SlicePreviewRenderResources>().unwrap();

        resources
            .slice_preview_pipeline
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
            .get::<SlicePreviewRenderResources>()
            .unwrap();

        resources.slice_preview_pipeline.paint(render_pass, self);
    }
}
