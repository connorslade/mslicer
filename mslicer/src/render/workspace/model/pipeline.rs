use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState,
    BufferBindingType, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState,
    DepthStencilState, Device, FragmentState, MultisampleState, PipelineLayoutDescriptor,
    RenderPipeline, RenderPipelineDescriptor, SamplerBindingType, ShaderStages, StencilFaceState,
    StencilState, TextureFormat, TextureSampleType, TextureViewDimension, VertexState,
};

use crate::{
    include_shader,
    render::{
        VERTEX_BUFFER_LAYOUT,
        consts::{BASE_BIND_GROUP_LAYOUT_DESCRIPTOR, DEPTH_STENCIL_STATE},
    },
};

pub fn pipeline(device: &Device, texture: TextureFormat) -> (RenderPipeline, BindGroupLayout) {
    let shader = device.create_shader_module(include_shader!("model.wgsl", "common.wgsl"));

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
            entry_point: None,
            buffers: &[VERTEX_BUFFER_LAYOUT],
            compilation_options: Default::default(),
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: None,
            targets: &[
                // target
                Some(ColorTargetState {
                    format: texture,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::all(),
                }),
                // world space
                Some(ColorTargetState {
                    format: TextureFormat::Rgba16Float,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::all(),
                }),
            ],
            compilation_options: Default::default(),
        }),
        primitive: Default::default(),
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
            stencil: StencilState {
                front: StencilFaceState::IGNORE,
                back: StencilFaceState::IGNORE,
                read_mask: 0,
                write_mask: 0,
            },
            bias: DepthBiasState {
                constant: 0,
                slope_scale: 0.0,
                clamp: 0.0,
            },
        }),
        multisample: MultisampleState {
            count: 4,
            ..Default::default()
        },
        multiview: None,
        cache: None,
    });

    (render_pipeline, bind_group_layout)
}

pub fn post_pipeline(device: &Device, texture: TextureFormat) -> (RenderPipeline, BindGroupLayout) {
    let post_shader =
        device.create_shader_module(include_shader!("model_post.wgsl", "common.wgsl"));

    let post_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX.union(ShaderStages::FRAGMENT),
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
                    sample_type: TextureSampleType::Float { filterable: false },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: false },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: true,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Depth,
                    view_dimension: TextureViewDimension::D2,
                    multisampled: true,
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

    let post_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&post_group_layout],
        push_constant_ranges: &[],
    });

    let post_render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: Some(&post_pipeline_layout),
        vertex: VertexState {
            module: &post_shader,
            entry_point: None,
            compilation_options: Default::default(),
            buffers: &[VERTEX_BUFFER_LAYOUT],
        },
        fragment: Some(FragmentState {
            module: &post_shader,
            entry_point: None,
            compilation_options: Default::default(),
            targets: &[Some(ColorTargetState {
                format: texture,
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::all(),
            })],
        }),
        primitive: Default::default(),
        depth_stencil: Some(DEPTH_STENCIL_STATE),
        multisample: MultisampleState {
            count: 4,
            ..Default::default()
        },
        multiview: None,
        cache: None,
    });

    (post_render_pipeline, post_group_layout)
}
