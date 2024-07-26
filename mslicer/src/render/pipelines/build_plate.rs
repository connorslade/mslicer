use eframe::Theme;
use egui_wgpu::ScreenDescriptor;
use encase::{ShaderSize, ShaderType, UniformBuffer};
use nalgebra::{Matrix4, Vector3, Vector4};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding, BufferBindingType,
    BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, CommandEncoder, CompareFunction,
    DepthStencilState, Device, FragmentState, IndexFormat, MultisampleState,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, Queue, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, TextureFormat,
    VertexState,
};

use crate::{
    include_shader,
    render::{workspace::WorkspaceRenderCallback, ModelVertex, VERTEX_BUFFER_LAYOUT},
    TEXTURE_FORMAT,
};

use super::Pipeline;

pub struct BuildPlatePipeline {
    render_pipeline: RenderPipeline,
    bind_group: BindGroup,

    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
    uniform_buffer: Buffer,

    last_bed_size: Vector3<f32>,
    last_grid_size: f32,
    vertex_count: u32,
}

#[derive(ShaderType)]
struct BuildPlateUniforms {
    transform: Matrix4<f32>,
    color: Vector4<f32>,
}

struct Line {
    start: Vector3<f32>,
    end: Vector3<f32>,
}

impl BuildPlatePipeline {
    pub fn new(device: &Device) -> Self {
        let shader = device.create_shader_module(include_shader!("solid.wgsl"));

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: BuildPlateUniforms::SHADER_SIZE.get(),
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

impl Pipeline<WorkspaceRenderCallback> for BuildPlatePipeline {
    fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        _screen_descriptor: &ScreenDescriptor,
        _encoder: &mut CommandEncoder,
        resources: &WorkspaceRenderCallback,
    ) {
        if self.last_bed_size != resources.bed_size || self.last_grid_size != resources.grid_size {
            let vertex = generate_mesh(resources.bed_size, resources.grid_size);
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

        let color = match resources.theme {
            Theme::Light => Vector4::new(0.0, 0.0, 0.0, 1.0),
            Theme::Dark => Vector4::new(1.0, 1.0, 1.0, 1.0),
        };

        let mut buffer = UniformBuffer::new(Vec::new());
        buffer
            .write(&BuildPlateUniforms {
                transform: resources.transform,
                color,
            })
            .unwrap();
        queue.write_buffer(&self.uniform_buffer, 0, &buffer.into_inner());
    }

    fn paint<'a>(&'a self, render_pass: &mut RenderPass<'a>, _resources: &WorkspaceRenderCallback) {
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
    fn new(start: Vector3<f32>, end: Vector3<f32>) -> Self {
        Self { start, end }
    }

    fn to_vertex(&self) -> [ModelVertex; 3] {
        [
            ModelVertex {
                position: [self.start.x, self.start.y, self.start.z, 1.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: [self.end.x, self.end.y, self.end.z, 1.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 0.0],
            },
            ModelVertex {
                position: [self.start.x, self.start.y, self.start.z, 1.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 0.0],
            },
        ]
    }
}

fn generate_mesh(bed_size: Vector3<f32>, grid_size: f32) -> Vec<ModelVertex> {
    let (a, b) = (bed_size / 2.0, -bed_size / 2.0);
    let z = bed_size.z;

    let mut lines = vec![
        // Bottom plane
        Line::new(Vector3::new(a.x, a.y, 0.0), Vector3::new(b.x, a.y, 0.0)),
        Line::new(Vector3::new(a.x, b.y, 0.0), Vector3::new(b.x, b.y, 0.0)),
        Line::new(Vector3::new(a.x, a.y, 0.0), Vector3::new(a.x, b.y, 0.0)),
        Line::new(Vector3::new(b.x, a.y, 0.0), Vector3::new(b.x, b.y, 0.0)),
        // Top plane
        Line::new(Vector3::new(a.x, a.y, z), Vector3::new(b.x, a.y, z)),
        Line::new(Vector3::new(a.x, b.y, z), Vector3::new(b.x, b.y, z)),
        Line::new(Vector3::new(a.x, a.y, z), Vector3::new(a.x, b.y, z)),
        Line::new(Vector3::new(b.x, a.y, z), Vector3::new(b.x, b.y, z)),
        // Vertical lines
        Line::new(Vector3::new(a.x, a.y, 0.0), Vector3::new(a.x, a.y, z)),
        Line::new(Vector3::new(b.x, a.y, 0.0), Vector3::new(b.x, a.y, z)),
        Line::new(Vector3::new(a.x, b.y, 0.0), Vector3::new(a.x, b.y, z)),
        Line::new(Vector3::new(b.x, b.y, 0.0), Vector3::new(b.x, b.y, z)),
    ];

    // Grid on bottom plane
    for x in 0..(bed_size.x / grid_size).ceil() as i32 {
        let x = x as f32 * grid_size + b.x;
        lines.push(Line::new(
            Vector3::new(x, a.y, 0.0),
            Vector3::new(x, b.y, 0.0),
        ));
    }

    for y in 0..(bed_size.y / grid_size).ceil() as i32 {
        let y = y as f32 * grid_size + b.y;
        lines.push(Line::new(
            Vector3::new(a.x, y, 0.0),
            Vector3::new(b.x, y, 0.0),
        ));
    }

    lines.iter().flat_map(Line::to_vertex).collect::<Vec<_>>()
}
