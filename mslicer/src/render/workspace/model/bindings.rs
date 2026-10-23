use nalgebra::Vector2;
use wgpu::{
    BufferUsages, Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    wgt::BufferDescriptor,
};

use crate::{
    core::App,
    render::{
        Gcx,
        workspace::model::{
            ModelPipeline, MultiStage, Target, TextureViews,
            pass::{
                base::BaseResources, blur::BlurResources, composite::CompositeResources,
                fxaa::FxaaResources, lighting::LightingResources, ssao::SsaoResources,
            },
        },
    },
};

impl ModelPipeline {
    pub fn size_textures(&mut self, gcx: &Gcx, app: &App, target: Target, size: Vector2<u32>) {
        let occlusion_size = match target {
            Target::Viewport => texture_size(app.config.render.ambient_occlusion.scale, size),
            Target::Preview => size,
        };

        let extent = Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        };
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

        let accumulation = gcx.device.create_buffer(&BufferDescriptor {
            label: Some("Accumulation"),
            size: extent.width as u64 * extent.height as u64 * 4,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

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

        let textures = TextureViews {
            target_a: target_a_view,
            target_b: target_b_view,
            accumulation,

            occlusion_target_a: occlusion_target_a_view,
            occlusion_target_b: occlusion_target_b_view,

            depth_target: depth_target_view,
            model_target: model_target_view,
            normal_target: normal_target_view,
        };

        // either create or update the multi stage resources
        if self.get_target(target).is_none() {
            *self.get_target_mut(target) = Some(MultiStage {
                base: BaseResources::new(gcx),
                ssao: SsaoResources::new(gcx),
                blur: BlurResources::new(gcx),
                lighting: LightingResources::new(gcx),
                fxaa: FxaaResources::new(gcx),
                composite: CompositeResources::new(gcx),

                textures: textures.clone(), // todo: uninit?
            })
        }

        let multi = match target {
            Target::Viewport => &mut self.viewport,
            Target::Preview => &mut self.preview,
        }
        .as_mut()
        .unwrap();

        let (sampler, bi_sampler) = (&self.sampler, &self.filtering_sampler);
        (multi.ssao).recreate_bind_group(gcx, &self.ssao, &textures, sampler);
        (multi.blur).recreate_bind_group(gcx, &self.blur, &textures, sampler);
        (multi.lighting).recreate_bind_group(gcx, &self.lighting, &textures, sampler, bi_sampler);
        (multi.fxaa).recreate_bind_group(gcx, &self.fxaa, &textures, sampler);
        (multi.composite).recreate_bind_group(gcx, &self.composite, &textures, sampler);
        multi.textures = textures;
    }
}

pub fn texture_size(scale: f32, size: Vector2<u32>) -> Vector2<u32> {
    (size.cast::<f32>() * scale.clamp(0.1, 1.0)).map(|x| x.ceil() as u32)
}
