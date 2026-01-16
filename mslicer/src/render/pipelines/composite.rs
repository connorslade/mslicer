use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
    Buffer, BufferUsages, ColorTargetState, ColorWrites, CompareFunction, DepthStencilState,
    Device, FilterMode, FragmentState, IndexFormat, MultisampleState, PipelineLayoutDescriptor,
    PrimitiveState, RenderPass, RenderPipeline, RenderPipelineDescriptor, Sampler,
    SamplerBindingType, ShaderStages, TextureFormat, TextureSampleType, TextureView,
    TextureViewDescriptor, TextureViewDimension, VertexState,
    util::{BufferInitDescriptor, DeviceExt},
    wgt::SamplerDescriptor,
};

use crate::{
    DEPTH_TEXTURE_FORMAT, include_shader,
    render::{Gcx, callback::WorkspaceRenderState, pipelines::consts::DEPTH_STENCIL_STATE},
};

pub struct CompositePipeline {
    render_pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,
    bind_group: Option<BindGroup>,

    sampler: Sampler,
    index_buffer: Buffer,
}

impl CompositePipeline {
    pub fn new(device: &Device, texture: TextureFormat) -> Self {
        let shader = device.create_shader_module(include_shader!("composite.wgsl"));
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: None,
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

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
                buffers: &[],
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
                format: DEPTH_TEXTURE_FORMAT,
                depth_write_enabled: false,
                depth_compare: CompareFunction::Always,
                ..DEPTH_STENCIL_STATE
            }),
            multisample: MultisampleState {
                count: 4,
                ..Default::default()
            },
            multiview: None,
            cache: None,
        });

        let index: [u16; 6] = [0, 1, 2, 2, 3, 0];
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&index),
            usage: BufferUsages::INDEX,
        });

        Self {
            render_pipeline,
            bind_group_layout,
            bind_group: None,

            sampler,
            index_buffer,
        }
    }
}

impl CompositePipeline {
    pub fn prepare(&mut self, gcx: &Gcx, solid: &TextureView) {
        self.bind_group = Some(gcx.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(&self.sampler),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&solid),
                },
            ],
        }));
    }

    pub fn paint(&self, render_pass: &mut RenderPass) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);

        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..1);
    }
}
