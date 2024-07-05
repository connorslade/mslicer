use egui_wgpu::ScreenDescriptor;
use encase::{ShaderSize, ShaderType, UniformBuffer};
use nalgebra::{Matrix4, Vector3};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, Buffer, BufferDescriptor, BufferUsages, ColorTargetState,
    ColorWrites, CommandEncoder, CompareFunction, DepthStencilState, Device, FragmentState,
    IndexFormat, MultisampleState, PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPass,
    RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, TextureFormat,
    VertexState,
};

use crate::{
    include_shader,
    render::{workspace::WorkspaceRenderCallback, ModelVertex, VERTEX_BUFFER_LAYOUT},
    TEXTURE_FORMAT,
};

use super::{
    consts::{BASE_BIND_GROUP_LAYOUT_DESCRIPTOR, BASE_UNIFORM_DESCRIPTOR},
    Pipeline,
};

pub struct TargetPointPipeline {
    render_pipeline: RenderPipeline,
    bind_group: BindGroup,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffer: Buffer,

    vertex_counts: u32,
}

#[derive(ShaderType)]
struct TargetPointUniforms {
    transform: Matrix4<f32>,
}

impl TargetPointPipeline {
    pub fn new(device: &Device) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(include_shader!("solid.wgsl").into()),
        });

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
            entries: &[],
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
                    format: TEXTURE_FORMAT,
                    blend: None,
                    write_mask: ColorWrites::all(),
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: MultisampleState {
                count: 4,
                ..Default::default()
            },
            multiview: None,
        });

        let tip = Vector3::new(0.0, 0.0, 1.0);
        let points = [
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(-1.0, 0.0, 0.0),
        ];

        let mut vertices = Vec::new();
        for i in 1..=4 {
            vertices.push(tip);
            vertices.push(points[i - 1]);
            vertices.push(points[i % 4]);

            vertices.push(-tip);
            vertices.push(-points[i - 1]);
            vertices.push(-points[i % 4]);
        }

        let vertices = vertices
            .into_iter()
            .map(|x| ModelVertex {
                position: [x.x, x.y, x.z, 1.0],
                ..Default::default()
            })
            .collect::<Vec<_>>();
        let vertex_counts = vertices.len() as u32;

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&(0..vertex_counts).collect::<Vec<_>>()),
            usage: BufferUsages::INDEX,
        });

        Self {
            render_pipeline,
            bind_group,

            vertex_buffer,
            index_buffer,
            uniform_buffer,

            vertex_counts,
        }
    }
}

impl Pipeline<WorkspaceRenderCallback> for TargetPointPipeline {
    fn prepare(
        &mut self,
        _device: &Device,
        queue: &Queue,
        _screen_descriptor: &ScreenDescriptor,
        _encoder: &mut CommandEncoder,
        resources: &WorkspaceRenderCallback,
    ) {
        let mut buffer = UniformBuffer::new(Vec::new());

        buffer
            .write(&TargetPointUniforms {
                transform: resources.transform * Matrix4::new_translation(&resources.target_point),
            })
            .unwrap();
        queue.write_buffer(&self.uniform_buffer, 0, &buffer.into_inner());
    }

    fn paint<'a>(&'a self, render_pass: &mut RenderPass<'a>, _resources: &WorkspaceRenderCallback) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.vertex_counts, 0, 0..1);
    }
}
