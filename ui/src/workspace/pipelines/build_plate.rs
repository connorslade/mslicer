use nalgebra::Vector3;
use wgpu::{
    util::DeviceExt, Buffer, ColorTargetState, ColorWrites, CompareFunction, DepthStencilState,
    Device, FragmentState, IndexFormat, MultisampleState, PipelineLayout, PrimitiveState,
    RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, TextureFormat,
    VertexState,
};

use crate::{
    include_shader,
    workspace::{
        render::{ModelVertex, VERTEX_BUFFER_LAYOUT},
        WorkspaceRenderCallback,
    },
    TEXTURE_FORMAT,
};

use super::Pipeline;

pub struct BuildPlatePipeline {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl BuildPlatePipeline {
    pub fn new(device: &Device, bed_size: Vector3<f32>) -> Self {
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
            vertex_buffer,
            index_buffer,
        }
    }
}

impl Pipeline for BuildPlatePipeline {
    fn init(&self, device: &Device, pipeline_layout: &PipelineLayout) -> RenderPipeline {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_shader!("build_plate.wgsl").into()),
        });

        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(pipeline_layout),
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
        })
    }

    fn paint<'a>(&'a self, render_pass: &mut RenderPass<'a>, _resources: &WorkspaceRenderCallback) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..6, 0, 0..1);
    }
}
