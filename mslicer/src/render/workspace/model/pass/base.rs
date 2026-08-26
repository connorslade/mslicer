use common::units::Milimeter;
use encase::{DynamicUniformBuffer, ShaderSize, ShaderType};
use nalgebra::{Matrix4, Vector3};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, BufferBinding,
    BufferBindingType, BufferUsages, Color, ColorTargetState, ColorWrites, CommandEncoder,
    CompareFunction, DepthBiasState, DepthStencilState, Device, FragmentState, IndexFormat, LoadOp,
    Operations, PipelineLayoutDescriptor, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, ShaderStages, StencilFaceState, StencilState, StoreOp, TextureFormat,
    VertexState,
};

use crate::{
    app::{
        App,
        camera::Camera,
        config::render::{Projection, RenderStyle},
    },
    include_shader,
    project::model::Model,
    render::{Gcx, VERTEX_BUFFER_LAYOUT, util::ResizingBuffer, workspace::model::MultiStage},
};

pub struct BasePass {
    pipeline: RenderPipeline,
    group_layout: BindGroupLayout,

    uniform: ResizingBuffer,
    offsets: Vec<u32>,
    bind_group: Option<BindGroup>,
}

#[derive(ShaderType)]
struct Uniforms {
    transform: Matrix4<f32>,
    model_transform: Matrix4<f32>,
    build_volume: Vector3<f32>,
    model_color: Vector3<f32>,
    render_style: u32,
    overhang_angle: f32,
}

impl BasePass {
    pub fn create(device: &Device, texture: TextureFormat) -> Self {
        let shader = device
            .create_shader_module(include_shader!("workspace/model/base.wgsl", "common.wgsl"));

        let group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(Uniforms::SHADER_SIZE),
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Model"),
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
                    // normals
                    Some(ColorTargetState {
                        format: TextureFormat::Rgba16Float,
                        blend: None,
                        write_mask: ColorWrites::all(),
                    }),
                    // world space
                    Some(ColorTargetState {
                        format: TextureFormat::Rgba32Float,
                        blend: None,
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
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });

        let uniform = ResizingBuffer::new(device, BufferUsages::UNIFORM | BufferUsages::COPY_DST);

        Self {
            pipeline,
            group_layout,
            uniform,
            offsets: Vec::new(),
            bind_group: None,
        }
    }

    fn write_uniforms(
        &mut self,
        gcx: &Gcx,
        app: &mut App,
        mut callback: impl FnMut(&Gcx, &mut Model) -> Uniforms,
    ) {
        self.offsets.clear();

        let mut buffer = DynamicUniformBuffer::new(Vec::new());
        for model in app.project.models.iter_mut().filter(|x| !x.hidden) {
            model.get_buffers(&gcx.device);

            let offset = buffer.write(&callback(gcx, model));
            self.offsets.push(offset.unwrap() as u32);
        }

        if self.uniform.write(gcx, &buffer.into_inner()) {
            let bind_group = gcx.device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &self.group_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &self.uniform,
                        offset: 0,
                        size: Some(Uniforms::SHADER_SIZE),
                    }),
                }],
            });

            self.bind_group.replace(bind_group);
        }
    }

    pub fn prepare(&mut self, gcx: &Gcx, app: &mut App) {
        let view_projection = app.view_projection();
        let render_style = app.config.render.style as u32;
        let build_volume = (app.project.slice_config)
            .platform_size
            .map(|x| x.get::<Milimeter>());

        let (show_overhang, overhang_angle) = app.config.render.overhangs;
        let overhang_angle = show_overhang.then_some(overhang_angle);
        let overhang_angle = overhang_angle
            .map(|x| x.to_radians())
            .unwrap_or(f32::from_bits(u32::MAX));

        self.write_uniforms(gcx, app, |gcx, model| {
            model.get_buffers(&gcx.device);
            let model_transform = *model.mesh.transformation_matrix();

            Uniforms {
                transform: view_projection * model_transform,
                model_transform,
                build_volume,
                model_color: model.color.to_srgb().into(),
                render_style,
                overhang_angle,
            }
        });
    }

    pub fn prepare_preview(&mut self, gcx: &Gcx, app: &mut App, camera: &Camera) {
        let view_projection = camera.view_projection_matrix(Projection::Perspective, 1.0);
        self.write_uniforms(gcx, app, |gcx, model| {
            model.get_buffers(&gcx.device);
            let model_transform = *model.mesh.transformation_matrix();

            Uniforms {
                transform: view_projection * model_transform,
                model_transform,
                build_volume: Vector3::repeat(f32::MAX),
                model_color: model.color.to_srgb().into(),
                render_style: RenderStyle::Rendered as u32,
                overhang_angle: 0.0,
            }
        });
    }

    pub fn paint(&self, encoder: &mut CommandEncoder, multi: &MultiStage, app: &mut App) {
        let Some(bind_group) = &self.bind_group else {
            return;
        };

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Model"),
            color_attachments: &[
                Some(RenderPassColorAttachment {
                    view: &multi.target_a,
                    resolve_target: None,
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
                Some(RenderPassColorAttachment {
                    view: &multi.world_target,
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

        render_pass.set_pipeline(&self.pipeline);

        let indexes = (app.project.models.iter().enumerate())
            .filter(|(_, x)| !x.hidden)
            .map(|(idx, _)| idx);

        for (i, idx) in indexes.enumerate() {
            render_pass.set_bind_group(0, bind_group, &[self.offsets[i]]);

            let model = &app.project.models[idx];
            let buffers = model.try_get_buffers().unwrap();
            render_pass.set_vertex_buffer(0, buffers.vertex_buffer.slice(..));
            render_pass.set_index_buffer(buffers.index_buffer.slice(..), IndexFormat::Uint32);
            render_pass.draw_indexed(0..(model.mesh.face_count() as u32 * 3), 0, 0..1);
        }
    }
}
