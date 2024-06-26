use std::mem;

use eframe::CreationContext;
use nalgebra::Vector3;
use pipelines::{
    build_plate::BuildPlatePipeline, model::ModelPipeline, slice_preview::SlicePreviewPipeline,
};
use slice_preview::SlicePreviewRenderResources;
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

use workspace::WorkspaceRenderResources;
pub mod camera;
pub mod pipelines;
pub mod rendered_mesh;
pub mod slice_preview;
pub mod workspace;

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

pub fn init_wgpu(cc: &CreationContext) {
    let render_state = cc.wgpu_render_state.as_ref().unwrap();
    let device = &render_state.device;

    let resources = &mut render_state.renderer.write().callback_resources;
    resources.insert(WorkspaceRenderResources {
        build_plate_pipeline: BuildPlatePipeline::new(device, Vector3::new(218.88, 122.904, 260.0)),
        model_pipeline: ModelPipeline::new(device),
    });
    resources.insert(SlicePreviewRenderResources {
        slice_preview_pipeline: SlicePreviewPipeline::new(device),
    });
}
