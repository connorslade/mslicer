use encase::{ShaderSize, ShaderType, UniformBuffer};
use nalgebra::{Matrix4, Vector3};
use wgpu::{
    BindGroup, BlendState, Buffer, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites,
    DepthStencilState, Device, FragmentState, IndexFormat, MultisampleState,
    PipelineLayoutDescriptor, PrimitiveState, RenderPass, RenderPipeline, RenderPipelineDescriptor,
    TextureFormat, VertexState,
};

use crate::{
    include_shader,
    render::{
        VERTEX_BUFFER_LAYOUT, gpu_mesh,
        pipelines::{
            ResizingBuffer,
            consts::{DEPTH_STENCIL_STATE, bind_group},
        },
        workspace::{Gcx, WorkspaceRenderCallback},
    },
};

use super::consts::{BASE_BIND_GROUP_LAYOUT_DESCRIPTOR, BASE_UNIFORM_DESCRIPTOR};

pub struct SupportPipeline {
    render_pipeline: RenderPipeline,
    bind_group: BindGroup,

    vertex_buffer: ResizingBuffer,
    index_buffer: ResizingBuffer,
    uniform_buffer: Buffer,
    index_count: u32,
}

#[derive(ShaderType)]
struct SupportUniforms {
    transform: Matrix4<f32>,
    camera_direction: Vector3<f32>,
}

impl SupportPipeline {
    pub fn new(device: &Device, texture: TextureFormat) -> Self {
        let shader = device.create_shader_module(include_shader!("support.wgsl", "common.wgsl"));

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            size: SupportUniforms::SHADER_SIZE.get(),
            ..BASE_UNIFORM_DESCRIPTOR
        });

        let (bind_group_layout, bind_group) = bind_group(
            device,
            BASE_BIND_GROUP_LAYOUT_DESCRIPTOR,
            [uniform_buffer.as_entire_binding()],
        );

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
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
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::all(),
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: Some(DepthStencilState {
                depth_write_enabled: false,
                ..DEPTH_STENCIL_STATE
            }),
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

            index_count: 0,
        }
    }
}

impl SupportPipeline {
    pub fn prepare(&mut self, gcx: &Gcx, resources: &WorkspaceRenderCallback) {
        let mesh = None;
        let Some(mesh) = mesh else {
            self.index_count = 0;
            return;
        };

        let (vertices, indices) = gpu_mesh(&mesh);
        self.vertex_buffer.write_slice(gcx, &vertices);
        self.index_buffer.write_slice(gcx, &indices);
        self.index_count = indices.len() as u32;

        let uniform = SupportUniforms {
            transform: resources.transform,
            camera_direction: -resources.camera.position() / resources.camera.distance,
        };

        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(&uniform).unwrap();
        gcx.queue
            .write_buffer(&self.uniform_buffer, 0, &buffer.into_inner());
    }

    pub fn paint(&self, render_pass: &mut RenderPass) {
        if self.index_count == 0 {
            return;
        }

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
