use egui_wgpu::ScreenDescriptor;
use encase::{ShaderType, UniformBuffer};
use nalgebra::Matrix4;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupEntry, BindGroupLayout, BufferUsages, ColorTargetState, ColorWrites,
    CommandEncoder, CompareFunction, DepthStencilState, Device, FragmentState, IndexFormat,
    MultisampleState, PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, TextureFormat, VertexState,
};

use crate::{
    include_shader,
    render::{
        rendered_mesh::RenderedMeshBuffers, workspace::WorkspaceRenderCallback,
        VERTEX_BUFFER_LAYOUT,
    },
    TEXTURE_FORMAT,
};

use super::{consts::BASE_BIND_GROUP_LAYOUT_DESCRIPTOR, Pipeline};

pub struct ModelPipeline {
    render_pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,

    bind_groups: Vec<BindGroup>,
}

#[derive(ShaderType)]
struct ModelUniforms {
    transform: Matrix4<f32>,
    model_transform: Matrix4<f32>,
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
        let shader = device.create_shader_module(include_shader!("model.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&BASE_BIND_GROUP_LAYOUT_DESCRIPTOR);

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
            bind_group_layout,
            bind_groups: Vec::new(),
        }
    }
}

impl Pipeline<WorkspaceRenderCallback> for ModelPipeline {
    fn prepare(
        &mut self,
        device: &Device,
        _queue: &Queue,
        _screen_descriptor: &ScreenDescriptor,
        _encoder: &mut CommandEncoder,
        resources: &WorkspaceRenderCallback,
    ) {
        self.bind_groups.clear();
        let mut to_generate = Vec::new();

        for (idx, model) in resources.models.read().unwrap().iter().enumerate() {
            if model.try_get_buffers().is_none() {
                to_generate.push(idx);
            }

            let model_transform = *model.mesh.transformation_matrix();
            let uniforms = ModelUniforms {
                transform: resources.transform * model_transform,
                model_transform,
                render_style: resources.render_style as u32,
            };

            let mut buffer = UniformBuffer::new(Vec::new());
            buffer.write(&uniforms).unwrap();

            let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: &buffer.into_inner(),
                usage: BufferUsages::UNIFORM,
            });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.bind_group_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
            });

            self.bind_groups.push(bind_group);
        }

        if !to_generate.is_empty() {
            let mut meshes = resources.models.write().unwrap();
            for idx in to_generate {
                meshes[idx].get_buffers(device);
            }
        }
    }

    fn paint<'a>(&'a self, render_pass: &mut RenderPass<'a>, resources: &WorkspaceRenderCallback) {
        render_pass.set_pipeline(&self.render_pipeline);

        let models = resources.models.read().unwrap();
        for (idx, model) in models.iter().enumerate().filter(|(_, x)| !x.hidden) {
            render_pass.set_bind_group(0, &self.bind_groups[idx], &[]);

            // SAFETY: im really tired and i dont care anymore
            let buffers =
                unsafe { &*(model.try_get_buffers().unwrap() as *const RenderedMeshBuffers) };
            render_pass.set_vertex_buffer(0, buffers.vertex_buffer.slice(..));
            render_pass.set_index_buffer(buffers.index_buffer.slice(..), IndexFormat::Uint32);
            render_pass.draw_indexed(0..(model.mesh.faces.len() as u32 * 3), 0, 0..1);
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
