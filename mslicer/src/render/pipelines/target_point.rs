use encase::{ShaderSize, ShaderType, UniformBuffer};
use nalgebra::{Matrix4, Vector4};
use plexus::primitive::{
    decompose::{Triangulate, Vertices},
    generate::{IndicesForPosition, VerticesWithPosition},
    sphere::UvSphere,
};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, BlendState, Buffer,
    BufferBinding, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, CompareFunction,
    DepthStencilState, Device, FragmentState, IndexFormat, MultisampleState,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, TextureFormat, VertexState,
};

use crate::{
    include_shader,
    render::{workspace::WorkspaceRenderCallback, ModelVertex, VERTEX_BUFFER_LAYOUT},
    DEPTH_TEXTURE_FORMAT,
};

use super::consts::{BASE_BIND_GROUP_LAYOUT_DESCRIPTOR, BASE_UNIFORM_DESCRIPTOR};

pub struct TargetPointPipeline {
    render_pipeline: RenderPipeline,
    bind_group: BindGroup,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffer: Buffer,

    index_count: u32,
}

#[derive(ShaderType)]
struct TargetPointUniforms {
    transform: Matrix4<f32>,
    color: Vector4<f32>,
}

impl TargetPointPipeline {
    pub fn new(device: &Device, texture: TextureFormat) -> Self {
        let shader = device.create_shader_module(include_shader!("solid.wgsl"));

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            size: TargetPointUniforms::SHADER_SIZE.get(),
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
                entry_point: "vert",
                buffers: &[VERTEX_BUFFER_LAYOUT],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "frag",
                targets: &[Some(ColorTargetState {
                    format: texture,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::all(),
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: Some(DepthStencilState {
                format: DEPTH_TEXTURE_FORMAT,
                depth_write_enabled: false,
                depth_compare: CompareFunction::Always,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: MultisampleState {
                count: 4,
                ..Default::default()
            },
            multiview: None,
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

        Self {
            render_pipeline,
            bind_group,

            vertex_buffer,
            index_buffer,
            uniform_buffer,

            index_count,
        }
    }
}

impl TargetPointPipeline {
    pub fn prepare(&mut self, queue: &Queue, resources: &WorkspaceRenderCallback) {
        if !resources.is_moving {
            return;
        };

        let mut buffer = UniformBuffer::new(Vec::new());

        buffer
            .write(&TargetPointUniforms {
                transform: resources.transform * Matrix4::new_translation(&resources.camera.target),
                color: Vector4::new(1.0, 0.0, 0.0, 0.25),
            })
            .unwrap();
        queue.write_buffer(&self.uniform_buffer, 0, &buffer.into_inner());
    }

    pub fn paint<'a>(
        &'a self,
        render_pass: &mut RenderPass<'a>,
        resources: &WorkspaceRenderCallback,
    ) {
        if !resources.is_moving {
            return;
        };

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}

fn generate_sphere(precision: usize) -> (Vec<ModelVertex>, Vec<u32>) {
    let sphere = UvSphere::new(precision, precision);

    let vertices = sphere
        .vertices_with_position()
        .map(|x| ModelVertex {
            position: [
                x.0.into_inner() as f32,
                x.1.into_inner() as f32,
                x.2.into_inner() as f32,
                1.0,
            ],
        })
        .collect();

    let indices = sphere
        .indices_for_position()
        .triangulate()
        .vertices()
        .map(|x| x as u32)
        .collect();

    (vertices, indices)
}
