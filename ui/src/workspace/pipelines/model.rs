use egui_wgpu::ScreenDescriptor;
use encase::{ShaderSize, ShaderType, UniformBuffer};
use nalgebra::Matrix4;
use wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, Buffer, BufferDescriptor, BufferUsages,
    ColorTargetState, ColorWrites, CommandEncoder, CompareFunction, DepthStencilState, Device,
    FragmentState, IndexFormat, MultisampleState, PipelineLayoutDescriptor, PrimitiveState, Queue,
    RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, TextureFormat,
    VertexState,
};

use crate::{
    include_shader,
    workspace::{
        rendered_mesh::RenderedMeshBuffers, WorkspaceRenderCallback, VERTEX_BUFFER_LAYOUT,
    },
    TEXTURE_FORMAT,
};

use super::Pipeline;

pub struct ModelPipeline {
    render_pipeline: RenderPipeline,
    bind_group: BindGroup,

    uniform_buffer: Buffer,
}

#[derive(ShaderType)]
struct ModelUniforms {
    transform: Matrix4<f32>,
    render_style: u32,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum RenderStyle {
    Normals,
    Rended,
}

impl ModelPipeline {
    pub fn new(device: &Device) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_shader!("model.wgsl").into()),
        });

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: ModelUniforms::SHADER_SIZE.get(),
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
                resource: uniform_buffer.as_entire_binding(),
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

        Self {
            render_pipeline,
            bind_group,
            uniform_buffer,
        }
    }
}

impl Pipeline for ModelPipeline {
    fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        _screen_descriptor: &ScreenDescriptor,
        _encoder: &mut CommandEncoder,
        resources: &WorkspaceRenderCallback,
    ) {
        let mut to_generate = Vec::new();
        for (idx, model) in resources.models.read().unwrap().iter().enumerate() {
            if model.try_get_buffers().is_none() {
                to_generate.push(idx);
            }
        }

        if !to_generate.is_empty() {
            let mut meshes = resources.models.write().unwrap();
            for idx in to_generate {
                meshes[idx].get_buffers(device);
            }
        }

        let uniforms = ModelUniforms {
            transform: resources.transform,
            render_style: resources.render_style as u8 as u32,
        };

        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(&uniforms).unwrap();
        queue.write_buffer(&self.uniform_buffer, 0, &buffer.into_inner());
    }

    fn paint<'a>(&'a self, render_pass: &mut RenderPass<'a>, resources: &WorkspaceRenderCallback) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);

        let models = resources.models.read().unwrap();
        for model in models.iter().filter(|x| !x.hidden) {
            // SAFETY: im really tired and i dont care anymore
            let buffers: &RenderedMeshBuffers =
                unsafe { &*(model.try_get_buffers().unwrap() as *const _) };
            render_pass.set_vertex_buffer(0, buffers.vertex_buffer.slice(..));
            render_pass.set_index_buffer(buffers.index_buffer.slice(..), IndexFormat::Uint32);
            render_pass.draw_indexed(0..(3 * model.mesh.faces.len() as u32), 0, 0..1);
        }
    }
}

impl RenderStyle {
    pub fn name(&self) -> &'static str {
        match self {
            RenderStyle::Normals => "Normals",
            RenderStyle::Rended => "Rended",
        }
    }
}
