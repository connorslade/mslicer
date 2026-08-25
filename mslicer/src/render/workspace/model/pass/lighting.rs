use encase::{ShaderSize, ShaderType, UniformBuffer};
use nalgebra::Vector3;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType,
    BufferDescriptor, BufferUsages, Color, ColorTargetState, ColorWrites, CommandEncoder, Device,
    FragmentState, IndexFormat, LoadOp, Operations, PipelineLayoutDescriptor,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    Sampler, SamplerBindingType, ShaderStages, StoreOp, TextureFormat, TextureSampleType,
    TextureViewDimension, VertexState,
};

use crate::{
    app::{App, camera::Camera},
    include_shader,
    render::{Gcx, workspace::model::MultiStage},
};

pub struct LightingPass {
    pipeline: RenderPipeline,
    group_layout: BindGroupLayout,
    uniform: Buffer,
    bind_group: Option<BindGroup>,
}

#[derive(ShaderType)]
struct Uniforms {
    camera_position: Vector3<f32>,
    flags: u32,
}

impl LightingPass {
    pub fn create(device: &Device, texture: TextureFormat) -> Self {
        let shader = device.create_shader_module(include_shader!(
            "workspace/model/lighting.wgsl",
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
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
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
                    blend: None,
                    write_mask: ColorWrites::all(),
                })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });

        let uniform = device.create_buffer(&BufferDescriptor {
            label: None,
            size: Uniforms::SHADER_SIZE.get(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            group_layout,
            uniform,
            bind_group: None,
        }
    }

    pub fn prepare(&mut self, gcx: &Gcx, app: &mut App, camera: Option<&Camera>) {
        let mut buffer = UniformBuffer::new(Vec::new());

        let ao = app.config.render.ambient_occlusion.enabled;
        let camera = camera.unwrap_or(&app.camera);

        let uniform = Uniforms {
            camera_position: camera.position(camera.distance),
            flags: ao as u32,
        };

        buffer.write(&uniform).unwrap();
        gcx.queue
            .write_buffer(&self.uniform, 0, &buffer.into_inner());
    }

    pub fn recreate_bind_group(&mut self, gcx: &Gcx, multi: &MultiStage, sampler: &Sampler) {
        self.bind_group
            .replace(gcx.device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &self.group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: self.uniform.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(&multi.target_a),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::TextureView(&multi.normal_target),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: BindingResource::TextureView(&multi.occlusion_target_b),
                    },
                    BindGroupEntry {
                        binding: 4,
                        resource: BindingResource::Sampler(sampler),
                    },
                ],
            }));
    }

    pub fn paint(&self, encoder: &mut CommandEncoder, multi: &MultiStage, index: &Buffer) {
        let Some(bind_group) = &self.bind_group else {
            return;
        };

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &multi.target_b,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::TRANSPARENT),
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.set_index_buffer(index.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..3, 0, 0..1);
    }
}
