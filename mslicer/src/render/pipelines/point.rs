use std::f32::consts::{PI, TAU};

use bytemuck::{Pod, Zeroable};
use encase::{ShaderSize, ShaderType, UniformBuffer};
use nalgebra::{Matrix4, Vector3, Vector4};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, BlendState, Buffer,
    BufferBinding, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, Device,
    FragmentState, IndexFormat, MultisampleState, PipelineLayoutDescriptor, PrimitiveState, Queue,
    RenderPass, RenderPipeline, RenderPipelineDescriptor, TextureFormat, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

use crate::{
    include_shader,
    render::{
        pipelines::consts::DEPTH_STENCIL_STATE, workspace::WorkspaceRenderCallback, ModelVertex,
        VERTEX_BUFFER_LAYOUT,
    },
};

use super::consts::{BASE_BIND_GROUP_LAYOUT_DESCRIPTOR, BASE_UNIFORM_DESCRIPTOR};

const INSTANCE_BUFFER_LAYOUT: VertexBufferLayout = VertexBufferLayout {
    array_stride: Point::SHADER_SIZE.get(),
    step_mode: VertexStepMode::Instance,
    attributes: &[
        VertexAttribute {
            format: VertexFormat::Float32x3,
            offset: 0,
            shader_location: 1,
        },
        VertexAttribute {
            format: VertexFormat::Float32,
            offset: 12,
            shader_location: 2,
        },
        VertexAttribute {
            format: VertexFormat::Float32x4,
            offset: 16,
            shader_location: 3,
        },
    ],
};

pub struct PointPipeline {
    render_pipeline: RenderPipeline,
    bind_group: BindGroup,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
    instance_buffer: Buffer,
    uniform_buffer: Buffer,

    index_count: u32,
    instance_count: u32,
}

#[derive(ShaderType)]
pub struct Point {
    pub position: Vector3<f32>,
    pub radius: f32,
    pub color: Vector4<f32>,
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct PointInstance {
    position: [f32; 3],
    radius: f32,
    color: [f32; 4],
}

#[derive(ShaderType)]
struct PointUniforms {
    transform: Matrix4<f32>,
}

impl PointPipeline {
    pub fn new(device: &Device, texture: TextureFormat) -> Self {
        let shader = device.create_shader_module(include_shader!("point.wgsl"));

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            size: PointUniforms::SHADER_SIZE.get(),
            ..BASE_UNIFORM_DESCRIPTOR
        });

        let bind_group_layout = device.create_bind_group_layout(&BASE_BIND_GROUP_LAYOUT_DESCRIPTOR);
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: None,
                buffers: &[VERTEX_BUFFER_LAYOUT, INSTANCE_BUFFER_LAYOUT],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: None,
                targets: &[Some(ColorTargetState {
                    format: texture,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::all(),
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: Some(DEPTH_STENCIL_STATE),
            multisample: MultisampleState {
                count: 4,
                ..Default::default()
            },
            multiview: None,
            cache: None,
        });

        let (vertices, indices) = generate_sphere(20);
        let index_count = indices.len() as u32;

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

        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: 0,
            usage: BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        Self {
            render_pipeline,
            bind_group,

            vertex_buffer,
            index_buffer,
            instance_buffer,
            uniform_buffer,

            index_count,
            instance_count: 0,
        }
    }
}

impl PointPipeline {
    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        resources: &WorkspaceRenderCallback,
        points: Option<&[&[Point]]>,
    ) {
        if let Some(points) = points {
            let points = (points.iter())
                .flat_map(|x| x.iter())
                .map(|x| x.to_instance())
                .collect::<Vec<_>>();

            self.instance_count = points.len() as u32;
            self.instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&points),
                usage: BufferUsages::VERTEX,
            });
        }

        let mut buffer = UniformBuffer::new(Vec::new());
        let transform = resources.transform;
        buffer.write(&PointUniforms { transform }).unwrap();
        queue.write_buffer(&self.uniform_buffer, 0, &buffer.into_inner());
    }

    pub fn paint(&self, render_pass: &mut RenderPass) {
        if self.instance_count == 0 {
            return;
        }

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..self.instance_count);
    }
}

impl Point {
    fn to_instance(&self) -> PointInstance {
        PointInstance {
            position: self.position.into(),
            radius: self.radius,
            color: self.color.into(),
        }
    }
}

/// Returns a unit sphere mesh with the specified number of vertices along the pitch and azimuth.
fn generate_sphere(precision: u32) -> (Vec<ModelVertex>, Vec<u32>) {
    let (mut vertices, mut indices) = (Vec::new(), Vec::new());
    for i_theta in 0..=precision {
        let theta = i_theta as f32 / precision as f32 * TAU;
        for i_phi in 0..=precision {
            let phi = i_phi as f32 / precision as f32 * PI;

            let idx = vertices.len() as u32;
            let rect = Vector3::new(phi.sin() * theta.cos(), phi.sin() * theta.sin(), phi.cos());
            vertices.push(ModelVertex {
                position: rect.push(1.0).into(),
            });

            if i_theta < precision && i_phi < precision {
                indices.extend([idx, idx + 1, idx + precision + 1]);
                indices.extend([idx + 1, idx + precision + 2, idx + precision + 1]);
            }
        }
    }

    (vertices, indices)
}
