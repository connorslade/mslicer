use nalgebra::Vector2;
use wgpu::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

use crate::render::{
    Gcx,
    workspace::model::{ModelPipeline, MultiStage},
};

impl ModelPipeline {
    pub(super) fn size_textures(&mut self, gcx: &Gcx, size: Vector2<u32>) -> bool {
        let extent = Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        };

        if let Some(multi_stage) = &self.multi_stage
            && multi_stage.target_a.texture().size() == extent
        {
            return false;
        }

        let target_desc = TextureDescriptor {
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
        };

        let target_a = gcx.device.create_texture(&target_desc);
        let target_b = gcx.device.create_texture(&target_desc);

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

        let normal_target = gcx.device.create_texture(&TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let world_target = gcx.device.create_texture(&TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let occlusion_target = gcx.device.create_texture(&TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R16Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let target_a_view = target_a.create_view(&Default::default());
        let target_b_view = target_b.create_view(&Default::default());
        let depth_target_view = depth_target.create_view(&Default::default());
        let normal_target_view = normal_target.create_view(&Default::default());
        let world_target_view = world_target.create_view(&Default::default());
        let occlusion_target_view = occlusion_target.create_view(&Default::default());

        self.multi_stage = Some(MultiStage {
            target_a: target_a_view,
            target_b: target_b_view,
            depth_target: depth_target_view,
            normal_target: normal_target_view,
            world_target: world_target_view,
            occlusion_target: occlusion_target_view,
        });
        true
    }
}
