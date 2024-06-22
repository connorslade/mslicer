use egui_wgpu::ScreenDescriptor;
use encase::{ShaderSize, ShaderType, UniformBuffer};
use nalgebra::{Matrix4, Vector3};
use wgpu::{
    util::DeviceExt, BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, Buffer, BufferBinding, BufferBindingType, BufferDescriptor, BufferUsages,
    ColorTargetState, ColorWrites, CommandEncoder, CompareFunction, DepthStencilState, Device,
    FragmentState, IndexFormat, MultisampleState, PipelineLayoutDescriptor, PrimitiveState, Queue,
    RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderStages,
    TextureFormat, VertexState,
};

use crate::{
    include_shader,
    workspace::{ModelVertex, WorkspaceRenderCallback, VERTEX_BUFFER_LAYOUT},
    TEXTURE_FORMAT,
};

use super::Pipeline;

pub struct BuildPlatePipeline {
    render_pipeline: RenderPipeline,
    bind_group: BindGroup,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffer: Buffer,
}

#[derive(ShaderType)]
struct BuildPlateUniforms {
    bed_size: Vector3<f32>,
    transform: Matrix4<f32>,
}

impl BuildPlatePipeline {
    pub fn new(device: &Device, bed_size: Vector3<f32>) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_shader!("build_plate.wgsl").into()),
        });

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
                ..Default::default()
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: MultisampleState::default(),
            multiview: None,
        });

        let (a, b) = (bed_size / 2.0, -bed_size / 2.0);

        let vert = [
            [a.x, a.y, 0.0],
            [b.x, a.y, 0.0],
            [b.x, b.y, 0.0],
            [b.x, b.y, 0.0],
            [a.x, b.y, 0.0],
            [a.x, a.y, 0.0],
        ]
        .into_iter()
        .map(|x| ModelVertex {
            position: [x[0], x[1], x[2], 1.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        })
        .collect::<Vec<_>>();
        let index: [u32; 6] = [0, 1, 2, 3, 4, 5];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Platform Vertex Buffer"),
            contents: bytemuck::cast_slice(&vert),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Platform Index Buffer"),
            contents: bytemuck::cast_slice(&index),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            render_pipeline,
            bind_group,

            vertex_buffer,
            index_buffer,
            uniform_buffer,
        }
    }
}

impl Pipeline for BuildPlatePipeline {
    fn prepare(
        &self,
        _device: &Device,
        queue: &Queue,
        _screen_descriptor: &ScreenDescriptor,
        _encoder: &mut CommandEncoder,
        resources: &WorkspaceRenderCallback,
    ) {
        let mut buffer = UniformBuffer::new(Vec::new());
        buffer
            .write(&BuildPlateUniforms {
                bed_size: resources.bed_size,
                transform: resources.transform,
            })
            .unwrap();
        queue.write_buffer(&self.uniform_buffer, 0, &buffer.into_inner());
    }

    fn paint<'a>(&'a self, render_pass: &mut RenderPass<'a>, _resources: &WorkspaceRenderCallback) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..6, 0, 0..1);
    }
}
