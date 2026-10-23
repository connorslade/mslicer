use encase::{ShaderSize, ShaderType, UniformBuffer};
use nalgebra::Vector2;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, Buffer, BufferBindingType,
    BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, CompareFunction,
    DepthStencilState, Device, FragmentState, IndexFormat, MultisampleState,
    PipelineLayoutDescriptor, RenderPass, RenderPipeline, RenderPipelineDescriptor, Sampler,
    SamplerBindingType, ShaderStages, TextureFormat, TextureSampleType, TextureViewDimension,
    VertexState,
};

use crate::{
    core::App,
    include_shader,
    render::{Gcx, consts::DEPTH_STENCIL_STATE, workspace::model::TextureViews},
};

pub struct CompositePass {
    pipeline: RenderPipeline,
    group_layout: BindGroupLayout,
}

pub struct CompositeResources {
    uniform: Buffer,
    bind_group: Option<BindGroup>,
}

#[derive(ShaderType)]
pub struct Uniforms {
    size: Vector2<u32>, // only send width. push constant?
    frame: u32,
    dirty: u32,
}

impl CompositePass {
    pub fn create(device: &Device, texture: TextureFormat) -> Self {
        let shader = device.create_shader_module(include_shader!(
            "workspace/model/composite.wgsl",
            "workspace/model/post.wgsl",
            "common.wgsl"
        ));

        let group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Depth,
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: None,
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: None,
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: texture,
                    blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: ColorWrites::all(),
                })],
            }),
            primitive: Default::default(),
            depth_stencil: Some(DepthStencilState {
                depth_compare: CompareFunction::LessEqual,
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
            pipeline,
            group_layout,
        }
    }

    pub fn paint(
        &self,
        render_pass: &mut RenderPass,
        index: &Buffer,
        resources: &CompositeResources,
    ) {
        let bind_group = resources.bind_group.as_ref().unwrap();

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.set_index_buffer(index.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..3, 0, 0..1);
    }
}

impl CompositeResources {
    pub fn new(gcx: &Gcx) -> Self {
        let uniform = gcx.device.create_buffer(&BufferDescriptor {
            label: None,
            size: Uniforms::SHADER_SIZE.get(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            uniform,
            bind_group: None,
        }
    }

    pub fn recreate_bind_group(
        &mut self,
        gcx: &Gcx,
        pass: &CompositePass,
        textures: &TextureViews,
        sampler: &Sampler,
    ) {
        let bind_group = gcx.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pass.group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.uniform.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&textures.target_a),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&textures.depth_target),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: textures.accumulation.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::Sampler(sampler),
                },
            ],
        });
        self.bind_group.replace(bind_group);
    }

    pub fn prepare(&mut self, gcx: &Gcx, app: &mut App, taa: bool, size: Vector2<u32>) {
        let mut buffer = UniformBuffer::new(Vec::new());
        let uniform = Uniforms {
            size,
            frame: 0,
            dirty: (!taa || app.state.workspace.is_moving) as u32,
        };
        buffer.write(&uniform).unwrap();
        gcx.queue
            .write_buffer(&self.uniform, 0, &buffer.into_inner());
    }
}
