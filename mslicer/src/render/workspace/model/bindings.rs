use nalgebra::Vector2;
use wgpu::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
};

use crate::{
    core::App,
    render::{
        Gcx,
        workspace::model::{ModelPipeline, MultiStage, Target, pass::base::BaseResources},
    },
};

impl ModelPipeline {
    pub fn size_textures(&mut self, gcx: &Gcx, app: &App, target: Target, size: Vector2<u32>) {
        let extent = Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        };

        let occlusion_size = occlusion_size(app, size);
        let occlusion_extent = Extent3d {
            width: occlusion_size.x,
            height: occlusion_size.y,
            depth_or_array_layers: 1,
        };

        if let Some(multi_stage) = &self.get_target(target)
            && multi_stage.target_a.texture().size() == extent
            && multi_stage.occlusion_target_a.texture().size() == occlusion_extent
        {
            return;
        }

        let target_desc = TextureDescriptor {
            label: Some("Color"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: gcx.texture,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC
                | TextureUsages::COPY_DST,
            view_formats: &[],
        };

        let target_a = gcx.device.create_texture(&target_desc);
        let target_b = gcx.device.create_texture(&target_desc);

        let depth_target = gcx.device.create_texture(&TextureDescriptor {
            label: Some("Depth"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let model_target = gcx.device.create_texture(&TextureDescriptor {
            label: Some("Model"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rg32Uint,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let normal_target = gcx.device.create_texture(&TextureDescriptor {
            label: Some("Normal"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let world_target = gcx.device.create_texture(&TextureDescriptor {
            label: Some("World"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let occlusion_desc = TextureDescriptor {
            label: Some("Occlusion"),
            size: occlusion_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R16Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let occlusion_target_a = gcx.device.create_texture(&occlusion_desc);
        let occlusion_target_b = gcx.device.create_texture(&occlusion_desc);

        let target_a_view = target_a.create_view(&Default::default());
        let target_b_view = target_b.create_view(&Default::default());
        let occlusion_target_a_view = occlusion_target_a.create_view(&Default::default());
        let occlusion_target_b_view = occlusion_target_b.create_view(&Default::default());
        let depth_target_view = depth_target.create_view(&Default::default());
        let model_target_view = model_target.create_view(&Default::default());
        let normal_target_view = normal_target.create_view(&Default::default());
        let world_target_view = world_target.create_view(&Default::default());

        let sampler = &self.sampler;
        let bi_sampler = &self.filtering_sampler;
        let textures = TextureViews {
            target_a: target_a_view,
            target_b: target_b_view,

            occlusion_target_a: occlusion_target_a_view,
            occlusion_target_b: occlusion_target_b_view,

            depth_target: depth_target_view,
            model_target: model_target_view,
            normal_target: normal_target_view,
            world_target: world_target_view,
        };

        *self.get_target_mut(target) = Some(MultiStage {
            base: BaseResources::new(&gcx.device),
            ssao: self.ssao.recreate_bind_group(gcx, &textures, sampler),
            blur: self.blur.recreate_bind_group(gcx, &textures, sampler),
            lighting: self.lighting.recreate_bind_group(gcx, &textures, sampler),
            fxaa: self.fxaa.recreate_bind_group(gcx, &textures, bi_sampler),
            composite: self.composite.recreate_bind_group(gcx, &textures, sampler),

            target_a: textures.target_a,
            target_b: textures.target_b,
            occlusion_target_a: textures.occlusion_target_a,
            occlusion_target_b: textures.occlusion_target_b,
            depth_target: textures.depth_target,
            model_target: textures.model_target,
            normal_target: textures.normal_target,
            world_target: textures.world_target,
        });
    }
}

pub fn occlusion_size(app: &App, size: Vector2<u32>) -> Vector2<u32> {
    let scale = app.config.render.ambient_occlusion.scale.clamp(0.1, 1.0);
    (size.cast::<f32>() * scale).map(|x| x.ceil() as u32)
}

pub struct TextureViews {
    pub target_a: TextureView,
    pub target_b: TextureView,

    pub occlusion_target_a: TextureView,
    pub occlusion_target_b: TextureView,

    pub depth_target: TextureView,
    pub model_target: TextureView,
    pub normal_target: TextureView,
    pub world_target: TextureView,
}
