use std::mem;

use dispatch::line::LineDispatch;
use eframe::CreationContext;
use egui_wgpu::RenderState;
use nalgebra::Vector4;
use pipelines::{model::ModelPipeline, slice_preview::SlicePreviewPipeline};
use slice_preview::SlicePreviewRenderResources;
use slicer::mesh::Mesh;
use wgpu::{
    Buffer, BufferAddress, BufferUsages, Device, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
};

use workspace::WorkspaceRenderResources;

use crate::render::dispatch::point::PointDispatch;
pub mod camera;
mod dispatch;
pub mod model;
pub mod pipelines;
pub mod preview;
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
        solid_line: LineDispatch::new(device, texture),
        model_pipeline: ModelPipeline::new(device, texture),
        point: PointDispatch::new(device, texture),
    });
    resources.insert(SlicePreviewRenderResources {
        slice_preview_pipeline: SlicePreviewPipeline::new(device, texture),
    });

    render_state.clone()
}

pub fn gpu_mesh(mesh: &Mesh) -> (Vec<ModelVertex>, Vec<u32>) {
    let index = mesh.faces().iter().flatten().copied().collect::<Vec<_>>();
    let vertices = (mesh.vertices().iter())
        .map(|vert| ModelVertex::new(vert.push(1.0)))
        .collect::<Vec<_>>();
    (vertices, index)
}

pub fn gpu_mesh_buffers(device: &Device, mesh: &Mesh) -> (Buffer, Buffer) {
    let (vertices, indices) = gpu_mesh(mesh);

    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&vertices),
        usage: BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&indices),
        usage: BufferUsages::INDEX,
    });

    (vertex_buffer, index_buffer)
}

impl ModelVertex {
    pub fn new(pos: Vector4<f32>) -> Self {
        Self {
            position: [pos.x, pos.y, pos.z, pos.w],
        }
    }
}
