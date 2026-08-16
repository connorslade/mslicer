use encase::UniformBuffer;
use nalgebra::Vector2;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, BufferUsages, Extent3d,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::render::{
    Gcx,
    workspace::model::{ModelPipeline, ModelUniforms, MultiStage},
};

impl ModelPipeline {
    pub(super) fn bind_group(&self, gcx: &Gcx, uniforms: ModelUniforms) -> BindGroup {
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
            sample_count: 4,
            dimension: TextureDimension::D2,
            format: gcx.texture,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let resolved_target = gcx.device.create_texture(&TextureDescriptor {
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
            sample_count: 4,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let world_target = gcx.device.create_texture(&TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 4,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let target_view = target.create_view(&Default::default());
        let resolved_target_view = resolved_target.create_view(&Default::default());
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
                    resource: BindingResource::TextureView(&resolved_target_view),
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
            resolved_target: resolved_target_view,
            depth_target: depth_target_view,
            world_target: world_target_view,
            post_bind_group,
        });
    }
}
