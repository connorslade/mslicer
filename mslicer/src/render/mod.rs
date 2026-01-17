use std::mem;

use eframe::CreationContext;
use egui_wgpu::RenderState;
use nalgebra::Vector4;
use wgpu::{
    BufferAddress, Device, Queue, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode,
};

use crate::render::{
    slice_preview::{SlicePreviewPipeline, SlicePreviewRenderResources},
    workspace::{
        WorkspaceRenderResources, line::LineDispatch, model::ModelPipeline, point::PointDispatch,
        support::SupportPipeline,
    },
};

pub mod camera;
mod consts;
pub mod preview;
pub mod slice_preview;
pub mod util;
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

pub struct Gcx<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
}

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
        model: ModelPipeline::new(device, texture),
        support: SupportPipeline::new(device, texture),
        point: PointDispatch::new(device, texture),
        solid_line: LineDispatch::new(device, texture),
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
