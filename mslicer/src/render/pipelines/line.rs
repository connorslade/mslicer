use encase::{ShaderSize, ShaderType, UniformBuffer};
use nalgebra::{Matrix4, Vector3};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer, BufferBinding,
    BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, Device, FragmentState,
    IndexFormat, MultisampleState, PipelineLayoutDescriptor, PolygonMode, PrimitiveState,
    RenderPass, RenderPipeline, RenderPipelineDescriptor, TextureFormat, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

use crate::{
    include_shader,
    render::{
        pipelines::{
            ResizingBuffer,
            consts::{
                BASE_BIND_GROUP_LAYOUT_DESCRIPTOR, BASE_UNIFORM_DESCRIPTOR, DEPTH_STENCIL_STATE,
            },
        },
        workspace::{Gcx, WorkspaceRenderCallback},
    },
};

const VERTEX_BUFFER_LAYOUT: VertexBufferLayout = VertexBufferLayout {
    array_stride: LineVertex::SHADER_SIZE.get(),
    step_mode: VertexStepMode::Vertex,
    attributes: &[
        VertexAttribute {
            format: VertexFormat::Float32x4,
            offset: 0,
            shader_location: 0,
        },
        VertexAttribute {
            format: VertexFormat::Float32x3,
            offset: 4 * 4,
            shader_location: 1,
        },
    ],
};

pub struct SolidLinePipeline {
    render_pipeline: RenderPipeline,
    bind_group: BindGroup,

    vertex_buffer: ResizingBuffer,
    index_buffer: ResizingBuffer,
    uniform_buffer: Buffer,

    vertex_count: u32,
}

#[derive(Clone)]
pub struct Line {
    start: Vector3<f32>,
    end: Vector3<f32>,

    color: Vector3<f32>,
}

#[derive(ShaderType)]
struct LineUniforms {
    transform: Matrix4<f32>,
}

#[repr(C)]
#[derive(Default, Copy, Clone, ShaderType, bytemuck::Pod, bytemuck::Zeroable)]
struct LineVertex {
    pub position: [f32; 4],
    pub color: [f32; 3],
}

impl SolidLinePipeline {
    pub fn new(device: &Device, texture: TextureFormat) -> Self {
        let shader = device.create_shader_module(include_shader!("line.wgsl"));

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            size: LineUniforms::SHADER_SIZE.get(),
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
                buffers: &[VERTEX_BUFFER_LAYOUT],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: None,
                targets: &[Some(ColorTargetState {
                    format: texture,
                    blend: None,
                    write_mask: ColorWrites::all(),
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                polygon_mode: PolygonMode::Line,
                ..Default::default()
            },
            depth_stencil: Some(DEPTH_STENCIL_STATE),
            multisample: MultisampleState {
                count: 4,
                ..Default::default()
            },
            multiview: None,
            cache: None,
        });

        Self {
            render_pipeline,
            bind_group,

            vertex_buffer: ResizingBuffer::new(device, BufferUsages::VERTEX),
            index_buffer: ResizingBuffer::new(device, BufferUsages::INDEX),
            uniform_buffer,

            vertex_count: 0,
        }
    }
}

impl SolidLinePipeline {
    pub fn prepare(
        &mut self,
        gcx: &Gcx,
        resources: &WorkspaceRenderCallback,
        lines: Option<&[&[Line]]>,
    ) {
        if let Some(lines) = lines {
            let vertex = (lines.iter())
                .flat_map(|x| x.iter())
                .flat_map(Line::to_vertex)
                .collect::<Vec<_>>();
            let index = (0..vertex.len() as u32).collect::<Vec<_>>();

            self.vertex_count = vertex.len() as u32;
            self.vertex_buffer.write_slice(gcx, &vertex);
            self.index_buffer.write_slice(gcx, &index);
        }

        let mut buffer = UniformBuffer::new(Vec::new());
        let transform = resources.transform;
        buffer.write(&LineUniforms { transform }).unwrap();
        gcx.queue
            .write_buffer(&self.uniform_buffer, 0, &buffer.into_inner());
    }

    pub fn paint(&self, render_pass: &mut RenderPass) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.vertex_count, 0, 0..1);
    }
}

impl Line {
    pub fn new(start: Vector3<f32>, end: Vector3<f32>) -> Self {
        Self {
            start,
            end,
            color: Vector3::zeros(),
        }
    }

    pub fn color(mut self, color: Vector3<f32>) -> Self {
        self.color = color;
        self
    }

    fn to_vertex(&self) -> [LineVertex; 3] {
        let color = [self.color.x, self.color.y, self.color.z];
        [
            LineVertex {
                position: [self.start.x, self.start.y, self.start.z, 1.0],
                color,
            },
            LineVertex {
                position: [self.end.x, self.end.y, self.end.z, 1.0],
                color,
            },
            LineVertex {
                position: [self.start.x, self.start.y, self.start.z, 1.0],
                color,
            },
        ]
    }
}
