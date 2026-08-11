use common::units::Milimeter;
use egui_wgpu::ScreenDescriptor;
use encase::{ShaderSize, ShaderType, UniformBuffer};
use nalgebra::{Matrix4, Vector2, Vector3};
use serde::{Deserialize, Serialize};
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
    Buffer, BufferBindingType, BufferDescriptor, BufferUsages, Color, ColorTargetState,
    ColorWrites, CommandEncoder, CompareFunction, DepthBiasState, DepthStencilState, Device,
    Extent3d, FilterMode, FragmentState, IndexFormat, LoadOp, MultisampleState, Operations,
    PipelineLayoutDescriptor, RenderPass, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
    StencilFaceState, StencilState, StoreOp, TextureDescriptor, TextureDimension, TextureFormat,
    TextureSampleType, TextureUsages, TextureView, TextureViewDimension, VertexState,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    app::App,
    include_shader,
    render::{
        Gcx, VERTEX_BUFFER_LAYOUT,
        camera::{Camera, Projection},
        consts::{BASE_BIND_GROUP_LAYOUT_DESCRIPTOR, DEPTH_STENCIL_STATE},
    },
};

pub struct ModelPipeline {
    render_pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,

    multi_stage: Option<MultiStage>,
    post_render_pipeline: RenderPipeline,
    post_group_layout: BindGroupLayout,
    post_uniform: Buffer,
    post_index_buffer: Buffer,
    sampler: Sampler,

    bind_groups: Vec<BindGroup>,
}

struct MultiStage {
    target: TextureView,
    depth_target: TextureView,
    normal_target: TextureView,
    resolved_target: TextureView,

    post_bind_group: BindGroup,
}

#[derive(ShaderType)]
struct ModelUniforms {
    transform: Matrix4<f32>,
    model_transform: Matrix4<f32>,
    build_volume: Vector3<f32>,
    model_color: Vector3<f32>,
    camera_position: Vector3<f32>,
    render_style: u32,
    overhang_angle: f32,
}

#[derive(ShaderType)]
struct PostUniforms {
    view: Matrix4<f32>,
    inv_view: Matrix4<f32>,
}

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum RenderStyle {
    Normals,
    RandomTriangle,
    Rendered,
}

impl ModelPipeline {
    pub fn new(device: &Device, texture: TextureFormat) -> Self {
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
                    Some(ColorTargetState {
                        format: texture,
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: ColorWrites::all(),
                    }),
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

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: None,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        let post_uniform = device.create_buffer(&BufferDescriptor {
            label: None,
            size: PostUniforms::SHADER_SIZE.get(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

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

        let post_index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[0, 1, 2]),
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
        });

        Self {
            render_pipeline,
            bind_group_layout,

            post_render_pipeline,
            post_group_layout,
            post_index_buffer,
            post_uniform,
            sampler,
            multi_stage: None,

            bind_groups: Vec::new(),
        }
    }

    fn bind_group(&self, gcx: &Gcx, uniforms: ModelUniforms) -> BindGroup {
        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(&uniforms).unwrap();

        let uniform_buffer = gcx.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: &buffer.into_inner(),
            usage: BufferUsages::UNIFORM,
        });

        gcx.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        })
    }

    fn post_bind_group(&mut self, device: &Device, texture: TextureFormat, size: Vector2<u32>) {
        let extent = Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        };

        if let Some(multi_stage) = &self.multi_stage
            && multi_stage.target.texture().size() == extent
        {
            return;
        }

        let target = device.create_texture(&TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 4,
            dimension: TextureDimension::D2,
            format: texture,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let depth_target = device.create_texture(&TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 4,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let normal_target = device.create_texture(&TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 4,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let resolved_target = device.create_texture(&TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: texture,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let target_view = target.create_view(&Default::default());
        let depth_target_view = depth_target.create_view(&Default::default());
        let normal_target_view = normal_target.create_view(&Default::default());
        let resolved_target_view = resolved_target.create_view(&Default::default());

        let post_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.post_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.post_uniform.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&resolved_target_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&normal_target_view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&depth_target_view),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        self.multi_stage = Some(MultiStage {
            target: target_view,
            depth_target: depth_target_view,
            normal_target: normal_target_view,
            resolved_target: resolved_target_view,
            post_bind_group,
        });
    }
}

impl ModelPipeline {
    pub fn prepare(
        &mut self,
        gcx: &Gcx,
        screen: &ScreenDescriptor,
        encoder: &mut CommandEncoder,
        texture: TextureFormat,
        app: &mut App,
    ) {
        let (show_overhang, overhang_angle) = app.config.overhang_visualization;
        let overhang_angle = show_overhang.then_some(overhang_angle);
        let view_projection = app.view_projection();

        self.bind_groups.clear();
        for model in app.project.models.iter_mut() {
            model.get_buffers(&gcx.device);

            let model_transform = *model.mesh.transformation_matrix();
            let overhang_angle = overhang_angle
                .map(|x| x.to_radians())
                .unwrap_or(f32::from_bits(u32::MAX));

            let build_volume = (app.project.slice_config)
                .platform_size
                .map(|x| x.get::<Milimeter>());

            let uniforms = ModelUniforms {
                transform: view_projection * model_transform,
                model_transform,
                build_volume,
                model_color: model.color.to_srgb().into(),
                camera_position: app.camera.position(app.camera.distance),
                render_style: app.config.render_style as u32,
                overhang_angle,
            };
            self.bind_groups.push(self.bind_group(gcx, uniforms));
        }

        let mut buffer = UniformBuffer::new(Vec::new());
        buffer
            .write(&PostUniforms {
                view: view_projection,
                inv_view: view_projection.try_inverse().unwrap(),
            })
            .unwrap();
        gcx.queue
            .write_buffer(&self.post_uniform, 0, &buffer.into_inner());

        self.post_bind_group(&gcx.device, texture, screen.size_in_pixels.into());
        let multi = self.multi_stage.as_ref().unwrap();
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[
                Some(RenderPassColorAttachment {
                    view: &multi.target,
                    resolve_target: Some(&multi.resolved_target),
                    ops: Operations {
                        load: LoadOp::Clear(Color::TRANSPARENT),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                }),
                Some(RenderPassColorAttachment {
                    view: &multi.normal_target,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::TRANSPARENT),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                }),
            ],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &multi.depth_target,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);

        let indexes = (app.project.models.iter().enumerate())
            .filter(|(_, x)| !x.hidden)
            .map(|(idx, _)| idx);

        for idx in indexes {
            render_pass.set_bind_group(0, &self.bind_groups[idx], &[]);

            let model = &app.project.models[idx];
            let buffers = model.try_get_buffers().unwrap();
            render_pass.set_vertex_buffer(0, buffers.vertex_buffer.slice(..));
            render_pass.set_index_buffer(buffers.index_buffer.slice(..), IndexFormat::Uint32);
            render_pass.draw_indexed(0..(model.mesh.face_count() as u32 * 3), 0, 0..1);
        }
    }

    pub fn prepare_preview(&mut self, gcx: &Gcx, app: &mut App, camera: Camera) {
        self.bind_groups.clear();
        for model in app.project.models.iter_mut() {
            model.get_buffers(&gcx.device);

            let view_projection = camera.view_projection_matrix(Projection::Perspective, 1.0);
            let model_transform = *model.mesh.transformation_matrix();
            let uniforms = ModelUniforms {
                transform: view_projection * model_transform,
                model_transform,
                build_volume: Vector3::repeat(f32::MAX),
                model_color: model.color.to_srgb().into(),
                camera_position: camera.position(camera.distance),
                render_style: RenderStyle::Rendered as u32,
                overhang_angle: 0.0,
            };
            self.bind_groups.push(self.bind_group(gcx, uniforms));
        }
    }

    pub fn paint(&self, render_pass: &mut RenderPass, _app: &mut App) {
        let multi = self.multi_stage.as_ref().unwrap();

        render_pass.set_pipeline(&self.post_render_pipeline);
        render_pass.set_bind_group(0, &multi.post_bind_group, &[]);
        render_pass.set_index_buffer(self.post_index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..3, 0, 0..1);
    }
}

impl RenderStyle {
    pub const ALL: [RenderStyle; 3] = [
        RenderStyle::Normals,
        RenderStyle::RandomTriangle,
        RenderStyle::Rendered,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            RenderStyle::Normals => "Normals",
            RenderStyle::RandomTriangle => "Triangles",
            RenderStyle::Rendered => "Rendered",
        }
    }
}
