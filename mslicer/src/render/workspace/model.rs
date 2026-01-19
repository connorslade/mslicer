use encase::{ShaderType, UniformBuffer};
use nalgebra::{Matrix4, Vector3};
use serde::{Deserialize, Serialize};
use wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayout, BlendState, BufferUsages, ColorTargetState,
    ColorWrites, Device, FragmentState, IndexFormat, MultisampleState, PipelineLayoutDescriptor,
    PrimitiveState, RenderPass, RenderPipeline, RenderPipelineDescriptor, TextureFormat,
    VertexState,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    app::App,
    include_shader,
    render::{
        Gcx, VERTEX_BUFFER_LAYOUT,
        consts::{BASE_BIND_GROUP_LAYOUT_DESCRIPTOR, DEPTH_STENCIL_STATE},
    },
};

pub struct ModelPipeline {
    render_pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,

    bind_groups: Vec<BindGroup>,
}

#[derive(ShaderType)]
struct ModelUniforms {
    transform: Matrix4<f32>,
    model_transform: Matrix4<f32>,
    build_volume: Vector3<f32>,
    model_color: Vector3<f32>,
    camera_position: Vector3<f32>,
    camera_target: Vector3<f32>,
    render_style: u32,
    overhang_angle: f32,
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
                targets: &[Some(ColorTargetState {
                    format: texture,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::all(),
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: Some(DEPTH_STENCIL_STATE),
            multisample: MultisampleState {
                count: 4,
                ..Default::default()
            },
            multiview: None,
            cache: None,
        });

        Self {
            render_pipeline,
            bind_group_layout,
            bind_groups: Vec::new(),
        }
    }
}

impl ModelPipeline {
    pub fn prepare(&mut self, gcx: &Gcx, app: &mut App) {
        let (show_overhang, overhang_angle) = app.config.overhang_visualization;
        let overhang_angle = show_overhang.then_some(overhang_angle);

        self.bind_groups.clear();
        let mut to_generate = Vec::new();

        for (idx, model) in app.project.models.iter().enumerate() {
            if model.try_get_buffers().is_none() {
                to_generate.push(idx);
            }

            let model_transform = *model.mesh.transformation_matrix();
            let overhang_angle = overhang_angle
                .map(|x| x.to_radians())
                .unwrap_or(f32::from_bits(u32::MAX));

            let uniforms = ModelUniforms {
                transform: app.view_projection() * model_transform,
                model_transform,
                build_volume: app.project.slice_config.platform_size,
                model_color: model.color.to_srgb().into(),
                camera_position: app.camera.position(),
                camera_target: app.camera.target,
                render_style: app.config.render_style as u32,
                overhang_angle,
            };

            let mut buffer = UniformBuffer::new(Vec::new());
            buffer.write(&uniforms).unwrap();

            let uniform_buffer = gcx.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: &buffer.into_inner(),
                usage: BufferUsages::UNIFORM,
            });

            let bind_group = gcx.device.create_bind_group(&wgpu::BindGroupDescriptor {
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
            for idx in to_generate {
                app.project.models[idx].get_buffers(gcx.device);
            }
        }
    }

    pub fn paint(&self, render_pass: &mut RenderPass, app: &mut App) {
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
