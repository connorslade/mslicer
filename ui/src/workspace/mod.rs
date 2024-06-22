use std::{
    mem,
    sync::{Arc, RwLock},
};

use eframe::CreationContext;
use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor};
use nalgebra::{Matrix4, Vector3};
use pipelines::{
    build_plate::BuildPlatePipeline,
    model::{ModelPipeline, RenderStyle},
    Pipeline,
};
use wgpu::{
    BufferAddress, CommandBuffer, CommandEncoder, Device, Queue, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexStepMode,
};

use rendered_mesh::RenderedMesh;
pub mod camera;
pub mod pipelines;
pub mod rendered_mesh;

pub const VERTEX_BUFFER_LAYOUT: VertexBufferLayout = VertexBufferLayout {
    array_stride: mem::size_of::<ModelVertex>() as BufferAddress,
    step_mode: VertexStepMode::Vertex,
    attributes: &[
        VertexAttribute {
            format: VertexFormat::Float32x4,
            offset: 0,
            shader_location: 0,
        },
        VertexAttribute {
            format: VertexFormat::Float32x2,
            offset: 4 * 4,
            shader_location: 1,
        },
        VertexAttribute {
            format: VertexFormat::Float32x3,
            offset: 4 * 4 + 4 * 2,
            shader_location: 2,
        },
    ],
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 4],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

pub struct WorkspaceRenderResources {
    build_plate_pipeline: BuildPlatePipeline,
    model_pipeline: ModelPipeline,
}

pub struct WorkspaceRenderCallback {
    pub bed_size: Vector3<f32>,
    pub transform: Matrix4<f32>,
    pub models: Arc<RwLock<Vec<RenderedMesh>>>,
    pub render_style: RenderStyle,
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
        let resources = resources.get::<WorkspaceRenderResources>().unwrap();

        resources
            .build_plate_pipeline
            .prepare(device, queue, screen_descriptor, encoder, self);
        resources
            .model_pipeline
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
            .get::<WorkspaceRenderResources>()
            .unwrap();

        resources.build_plate_pipeline.paint(render_pass, self);
        resources.model_pipeline.paint(render_pass, self);
    }
}

pub fn init_wgpu(cc: &CreationContext) {
    let render_state = cc.wgpu_render_state.as_ref().unwrap();
    let device = &render_state.device;

    render_state
        .renderer
        .write()
        .callback_resources
        .insert(WorkspaceRenderResources {
            build_plate_pipeline: BuildPlatePipeline::new(
                device,
                Vector3::new(218.88, 122.904, 260.0),
            ),
            model_pipeline: ModelPipeline::new(device),
        });
}
