use encase::{ShaderSize, UniformBuffer};
use nalgebra::Vector2;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Buffer,
    BufferDescriptor, BufferUsages, Extent3d, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages,
};

use crate::render::{
    Gcx,
    workspace::model::{ModelPipeline, ModelUniforms, MultiStage},
};

pub struct ModelBindings {
    pub uniform: Buffer,
    pub bind_group: BindGroup,
}

impl ModelPipeline {
    pub(super) fn size_textures(&mut self, gcx: &Gcx, size: Vector2<u32>) {
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

        let target = gcx.device.create_texture(&TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: gcx.texture,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let depth_target = gcx.device.create_texture(&TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let world_target = gcx.device.create_texture(&TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let target_view = target.create_view(&Default::default());
        let depth_target_view = depth_target.create_view(&Default::default());
        let world_target_view = world_target.create_view(&Default::default());

        let post_bind_group = gcx.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.post_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.post_uniform.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&target_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&world_target_view),
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
            world_target: world_target_view,
            post_bind_group,
        });
    }
}

impl ModelBindings {
    pub fn create(gcx: &Gcx, layout: &BindGroupLayout) -> Self {
        let uniform = gcx.device.create_buffer(&BufferDescriptor {
            label: None,
            size: ModelUniforms::SHADER_SIZE.get(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = gcx.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });

        Self {
            uniform,
            bind_group,
        }
    }

    pub fn upload(&self, gcx: &Gcx, uniforms: &ModelUniforms) {
        let mut data = UniformBuffer::new(Vec::new());
        data.write(&uniforms).unwrap();
        gcx.queue.write_buffer(&self.uniform, 0, &data.into_inner());
    }
}
