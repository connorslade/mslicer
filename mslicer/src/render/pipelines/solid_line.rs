use std::mem;

use encase::{ShaderSize, ShaderType, UniformBuffer};
use nalgebra::{Matrix4, Vector3};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferAddress, BufferBinding,
    BufferBindingType, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites,
    CompareFunction, DepthStencilState, Device, FragmentState, IndexFormat, MultisampleState,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, Queue, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, TextureFormat,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

use crate::{include_shader, render::workspace::WorkspaceRenderCallback, TEXTURE_FORMAT};

pub struct SolidLinePipeline {
    render_pipeline: RenderPipeline,
    bind_group: BindGroup,

    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
    uniform_buffer: Buffer,

    last_bed_size: Vector3<f32>,
    last_grid_size: f32,
    vertex_count: u32,
}

#[derive(Clone)]
pub struct Line {
    start: Vector3<f32>,
    end: Vector3<f32>,

    color: Vector3<f32>,
}

#[derive(ShaderType)]
struct SolidLineUniforms {
    transform: Matrix4<f32>,
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct LineVertex {
    pub position: [f32; 4],
    pub color: [f32; 3],
}

impl SolidLinePipeline {
    pub fn new(device: &Device) -> Self {
        let shader = device.create_shader_module(include_shader!("solid_line.wgsl"));

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: SolidLineUniforms::SHADER_SIZE.get(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
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
                buffers: &[VertexBufferLayout {
                    array_stride: mem::size_of::<LineVertex>() as BufferAddress,
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
                }],
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
            primitive: PrimitiveState {
                polygon_mode: PolygonMode::Line,
                ..Default::default()
            },
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

        Self {
            render_pipeline,
            bind_group,

            vertex_buffer: None,
            index_buffer: None,
            uniform_buffer,

            last_bed_size: Vector3::zeros(),
            last_grid_size: 0.0,
            vertex_count: 0,
        }
    }
}

impl SolidLinePipeline {
    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        resources: &WorkspaceRenderCallback,
        lines: Option<&[&[Line]]>,
    ) {
        if let Some(lines) = lines {
            let vertex = lines
                .iter()
                .flat_map(|x| x.iter())
                .flat_map(Line::to_vertex)
                .collect::<Vec<_>>();
            self.vertex_count = vertex.len() as u32;
            self.last_bed_size = resources.bed_size;
            self.last_grid_size = resources.grid_size;

            self.vertex_buffer = Some(device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&vertex),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            }));

            self.index_buffer = Some(device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&(0..vertex.len() as u32).collect::<Vec<_>>()),
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            }));
        }

        let mut buffer = UniformBuffer::new(Vec::new());
        buffer
            .write(&SolidLineUniforms {
                transform: resources.transform,
            })
            .unwrap();
        queue.write_buffer(&self.uniform_buffer, 0, &buffer.into_inner());
    }

    pub fn paint<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
        render_pass.set_index_buffer(
            self.index_buffer.as_ref().unwrap().slice(..),
            IndexFormat::Uint32,
        );
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
