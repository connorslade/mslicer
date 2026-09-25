use eframe::CreationContext;
use egui_wgpu::RenderState;
use wgpu::{
    Device, Queue, TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode,
};

use crate::render::{
    interface::basis::BasisPipeline,
    slice_preview::{SlicePreviewPipeline, SlicePreviewRenderResources},
    workspace::{
        WorkspaceRenderResources, line::LineDispatch, model::ModelPipeline, point::PointDispatch,
        support::SupportPipeline,
    },
};

pub mod camera;
pub mod consts;
pub mod interface;
pub mod slice_preview;
pub mod util;
pub mod workspace;

pub const VERTEX_BUFFER_LAYOUT: VertexBufferLayout = VertexBufferLayout {
    array_stride: 4 * 3,
    step_mode: VertexStepMode::Vertex,
    attributes: &[VertexAttribute {
        format: VertexFormat::Float32x3,
        offset: 0,
        shader_location: 0,
    }],
};

pub struct Gcx {
    pub device: Device,
    pub queue: Queue,
    pub texture: TextureFormat,
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3], // implicit w=1
}

pub fn init_wgpu(cc: &CreationContext) -> RenderState {
    let render_state = cc.wgpu_render_state.as_ref().unwrap();
    let device = &render_state.device;
    let texture = render_state.target_format;

    let resources = &mut render_state.renderer.write().callback_resources;
    resources.insert(WorkspaceRenderResources {
        model: ModelPipeline::new(device, texture),
        support: SupportPipeline::new(device, texture),
        point: PointDispatch::new(device, texture),
        solid_line: LineDispatch::new(device, texture),
    });
    resources.insert(SlicePreviewRenderResources {
        slice_preview_pipeline: SlicePreviewPipeline::new(device, texture),
    });
    resources.insert(BasisPipeline::new(device, texture));

    render_state.clone()
}
