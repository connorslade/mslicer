use std::mem;

use eframe::CreationContext;
use nalgebra::Vector3;
use wgpu::{
    BindGroupEntry, BindGroupLayoutDescriptor, BufferAddress, BufferBinding, BufferDescriptor,
    BufferUsages, PipelineLayoutDescriptor, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexStepMode,
};

use super::pipelines::{build_plate::BuildPlatePipeline, model::ModelPipeline, CachedPipeline};
use crate::workspace::{WorkspaceRenderCallback, WorkspaceRenderResources};

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

    let uniform_buffer = device.create_buffer(&BufferDescriptor {
        label: None,
        size: mem::size_of::<WorkspaceRenderCallback>() as u64 + 64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(BufferBinding {
                buffer: &uniform_buffer,
                offset: 0,
                size: None,
            }),
        }],
    });

    render_state
        .renderer
        .write()
        .callback_resources
        .insert(WorkspaceRenderResources {
            uniform_buffer,
            bind_group,

            build_plate_pipeline: CachedPipeline::new(BuildPlatePipeline::new(
                device,
                Vector3::new(218.88, 122.904, 260.0),
            ))
            .prepare(device, &pipeline_layout),
            model_pipeline: CachedPipeline::new(ModelPipeline::new())
                .prepare(device, &pipeline_layout),
        });
}
