use std::mem;

use dispatch::solid_line::SolidLineDispatch;
use eframe::CreationContext;
use egui_wgpu::RenderState;
use nalgebra::Vector4;
use pipelines::{
    model::ModelPipeline, slice_preview::SlicePreviewPipeline, target_point::TargetPointPipeline,
};
use slice_preview::SlicePreviewRenderResources;
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

use workspace::WorkspaceRenderResources;
pub mod camera;
mod dispatch;
pub mod pipelines;
pub mod preview;
pub mod rendered_mesh;
pub mod slice_preview;
pub mod workspace;

pub const VERTEX_BUFFER_LAYOUT: VertexBufferLayout = VertexBufferLayout {
    array_stride: mem::size_of::<ModelVertex>() as BufferAddress,
    step_mode: VertexStepMode::Vertex,
    attributes: &[VertexAttribute {
        format: VertexFormat::Float32x4,
        offset: 0,
        shader_location: 0,
    }],
};

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 4],
}

pub fn init_wgpu(cc: &CreationContext) -> RenderState {
    let render_state = cc.wgpu_render_state.as_ref().unwrap();
    let device = &render_state.device;
    let texture = render_state.target_format;

    let resources = &mut render_state.renderer.write().callback_resources;
    resources.insert(WorkspaceRenderResources {
        solid_line: SolidLineDispatch::new(device, texture),
        model_pipeline: ModelPipeline::new(device, texture),
        target_point_pipeline: TargetPointPipeline::new(device, texture),
    });
    resources.insert(SlicePreviewRenderResources {
        slice_preview_pipeline: SlicePreviewPipeline::new(device, texture),
    });

    render_state.clone()
}

impl ModelVertex {
    pub fn new(pos: Vector4<f32>) -> Self {
        Self {
            position: [pos.x, pos.y, pos.z, pos.w],
        }
    }
}
