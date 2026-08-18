use common::units::Milimeter;
use egui_wgpu::ScreenDescriptor;
use encase::{ShaderSize, ShaderType, UniformBuffer};
use nalgebra::{Matrix4, Vector3};
use rand::random;
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, BufferDescriptor, BufferUsages, Color, CommandEncoder,
    Device, IndexFormat, LoadOp, Operations, RenderPass, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline, Sampler, StoreOp,
    TextureFormat, TextureView,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    app::App,
    render::{Gcx, consts::FILTERING_SAMPLER},
};

mod bindings;
mod pipeline;
mod preview;
pub use preview::process_previews;

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
    world_target: TextureView,

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
    samples: u32,
    random: u32,
    range: f32,
}

impl ModelPipeline {
    pub fn new(device: &Device, texture: TextureFormat) -> Self {
        let (render_pipeline, bind_group_layout) = pipeline::pipeline(device, texture);
        let (post_render_pipeline, post_group_layout) = pipeline::post_pipeline(device, texture);

        let sampler = device.create_sampler(&FILTERING_SAMPLER);

        let post_uniform = device.create_buffer(&BufferDescriptor {
            label: None,
            size: PostUniforms::SHADER_SIZE.get(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
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

    fn upload_uniforms(&mut self, gcx: &Gcx, app: &mut App) {
        let view_projection = app.view_projection();
        let build_volume = (app.project.slice_config)
            .platform_size
            .map(|x| x.get::<Milimeter>());

        let (show_overhang, overhang_angle) = app.config.render.overhangs;
        let overhang_angle = show_overhang.then_some(overhang_angle);
        let overhang_angle = overhang_angle
            .map(|x| x.to_radians())
            .unwrap_or(f32::from_bits(u32::MAX));

        self.bind_groups.clear();
        for model in app.project.models.iter_mut() {
            model.get_buffers(&gcx.device);

            let model_transform = *model.mesh.transformation_matrix();
            let uniforms = ModelUniforms {
                transform: view_projection * model_transform,
                model_transform,
                build_volume,
                model_color: model.color.to_srgb().into(),
                camera_position: app.camera.position(app.camera.distance),
                render_style: app.config.render.style as u32,
                overhang_angle,
            };
            self.bind_groups.push(self.bind_group(gcx, uniforms));
        }

        let ao = &app.config.render.ambient_occlusion;
        let post_uniform = PostUniforms {
            view: view_projection,
            random: random(),
            samples: ao.samples,
            range: ao.range,
        };

        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(&post_uniform).unwrap();
        gcx.queue
            .write_buffer(&self.post_uniform, 0, &buffer.into_inner());
    }

    fn render(&mut self, encoder: &mut CommandEncoder, app: &mut App) {
        let multi = self.multi_stage.as_ref().unwrap();
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[
                Some(RenderPassColorAttachment {
                    view: &multi.target,
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

    pub fn prepare(
        &mut self,
        gcx: &Gcx,
        screen: &ScreenDescriptor,
        encoder: &mut CommandEncoder,
        app: &mut App,
    ) {
        self.upload_uniforms(gcx, app);
        self.size_textures(gcx, screen.size_in_pixels.into());
        self.render(encoder, app);
    }

    // Runs the post-processing pipeline, copying the data from the intermediary
    // buffers to the output surface.
    pub fn paint(&self, render_pass: &mut RenderPass, _app: &mut App) {
        let multi = self.multi_stage.as_ref().unwrap();

        render_pass.set_pipeline(&self.post_render_pipeline);
        render_pass.set_bind_group(0, &multi.post_bind_group, &[]);
        render_pass.set_index_buffer(self.post_index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..3, 0, 0..1);
    }
}
