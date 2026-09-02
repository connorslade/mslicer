use egui_wgpu::ScreenDescriptor;
use wgpu::{
    Buffer, BufferUsages, CommandEncoder, Device, Origin3d, RenderPass, Sampler,
    TexelCopyTextureInfo, TextureAspect, TextureFormat, TextureView,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    app::App,
    render::{
        Gcx,
        consts::NONFILTERING_SAMPLER,
        workspace::model::pass::{
            base::BasePass, blur::BlurPass, composite::CompositePass, fxaa::FxaaPass,
            lighting::LightingPass, ssao::SsaoPass,
        },
    },
};

mod bindings;
mod pass;
mod preview;
pub use preview::process_previews;

pub struct ModelPipeline {
    multi_stage: Option<MultiStage>,
    base: BasePass,
    ssao: SsaoPass,
    blur: BlurPass,
    lighting: LightingPass,
    fxaa: FxaaPass,
    composite: CompositePass,

    post_index_buffer: Buffer,
    sampler: Sampler,
}

struct MultiStage {
    target_a: TextureView,
    target_b: TextureView,

    occlusion_target_a: TextureView,
    occlusion_target_b: TextureView,

    // g buffer
    depth_target: TextureView,
    normal_target: TextureView,
    world_target: TextureView,
}

impl ModelPipeline {
    pub fn new(device: &Device, texture: TextureFormat) -> Self {
        let sampler = device.create_sampler(&NONFILTERING_SAMPLER);

        let post_index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[0, 1, 2]),
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
        });

        Self {
            multi_stage: None,
            base: BasePass::create(device, texture),
            ssao: SsaoPass::create(device),
            blur: BlurPass::create(device),
            lighting: LightingPass::create(device, texture),
            fxaa: FxaaPass::create(device, texture),
            composite: CompositePass::create(device, texture),

            post_index_buffer,
            sampler,
        }
    }

    fn render(&mut self, encoder: &mut CommandEncoder, app: &mut App) {
        let multi = self.multi_stage.as_ref().unwrap();
        let index = &self.post_index_buffer;

        self.base.paint(encoder, multi, app);
        if app.config.render.ambient_occlusion.enabled {
            self.ssao.paint(encoder, multi, index);
            self.blur.paint(encoder, multi, index);
        }

        self.lighting.paint(encoder, multi, index);
        if app.config.render.anti_aliasing.enabled {
            self.fxaa.paint(encoder, multi, index);
        } else {
            encoder.copy_texture_to_texture(
                TexelCopyTextureInfo {
                    texture: multi.target_b.texture(),
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                TexelCopyTextureInfo {
                    texture: multi.target_a.texture(),
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                multi.target_a.texture().size(),
            );
        }
    }

    pub fn prepare(
        &mut self,
        gcx: &Gcx,
        screen: &ScreenDescriptor,
        encoder: &mut CommandEncoder,
        app: &mut App,
    ) {
        let size = screen.size_in_pixels.into();

        self.base.prepare(gcx, app);
        self.ssao.prepare(gcx, app, None);
        self.blur.prepare(gcx, app, size);
        self.lighting.prepare(gcx, app, None);
        self.fxaa.prepare(gcx, app, size);

        self.size_textures(gcx, app, screen.size_in_pixels.into());
        self.render(encoder, app);
    }

    // Runs the post-processing pipeline, copying the data from the intermediary
    // buffers to the output surface.
    pub fn paint(&self, render_pass: &mut RenderPass, _app: &mut App) {
        self.composite.paint(render_pass, &self.post_index_buffer);
    }
}
