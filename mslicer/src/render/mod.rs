use std::mem;

use eframe::CreationContext;
use egui_wgpu::RenderState;
use nalgebra::Vector4;
use slicer::mesh::Mesh;
use wgpu::{
    Buffer, BufferAddress, BufferUsages, Device, Extent3d, Queue, Texture, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, VertexAttribute, VertexBufferLayout,
    VertexFormat, VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    DEPTH_TEXTURE_FORMAT,
    render::{
        slice_preview::{SlicePreviewPipeline, SlicePreviewRenderResources},
        workspace::{
            WorkspaceRenderResources, line::LineDispatch, model::ModelPipeline,
            point::PointDispatch, support::SupportPipeline,
        },
    },
};

pub mod camera;
mod consts;
pub mod model;
pub mod preview;
pub mod slice_preview;
mod util;
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

pub fn init_textures(
    device: &Device,
    format: TextureFormat,
    (width, height): (u32, u32),
) -> (Texture, Texture, Texture) {
    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 4,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    let resolved_texture = device.create_texture(&TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::RENDER_ATTACHMENT
            | TextureUsages::COPY_SRC
            | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let depth_texture = device.create_texture(&TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 4,
        dimension: TextureDimension::D2,
        format: DEPTH_TEXTURE_FORMAT,
        usage: TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    (texture, resolved_texture, depth_texture)
}

impl ModelVertex {
    pub fn new(pos: Vector4<f32>) -> Self {
        Self {
            position: [pos.x, pos.y, pos.z, pos.w],
        }
    }
}
