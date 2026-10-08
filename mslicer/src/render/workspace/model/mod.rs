use common::color::SRgb;
use egui_wgpu::ScreenDescriptor;
use wgpu::{
    BindGroup, Buffer, BufferUsages, CommandEncoder, Device, Origin3d, RenderPass, Sampler,
    TexelCopyTextureInfo, TextureAspect, TextureFormat, TextureView,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    core::App,
    render::{
        Gcx,
        consts::{FILTERING_SAMPLER, NONFILTERING_SAMPLER},
        workspace::model::{
            bindings::occlusion_size,
            pass::{
                base::BasePass, blur::BlurPass, composite::CompositePass, fxaa::FxaaPass,
                lighting::LightingPass, ssao::SsaoPass,
            },
            pick::ModelPicker,
        },
    },
};

mod bindings;
mod pass;
mod pick;
mod preview;
mod selected;
pub use preview::process_previews;

pub struct ModelPipeline {
    viewport: Option<MultiStage>,
    preview: Option<MultiStage>,

    picker: ModelPicker,
    base: BasePass,
    ssao: SsaoPass,
    blur: BlurPass,
    lighting: LightingPass,
    fxaa: FxaaPass,
    composite: CompositePass,

    post_index_buffer: Buffer,
    sampler: Sampler,
    filtering_sampler: Sampler,
}

#[derive(Clone, Copy)]
pub enum Target {
    Viewport,
    Preview,
}

struct MultiStage {
    target_a: TextureView,
    target_b: TextureView,

    occlusion_target_a: TextureView,
    occlusion_target_b: TextureView,

    // g buffer
    depth_target: TextureView,
    model_target: TextureView,
    normal_target: TextureView,
    world_target: TextureView,

    //bind groups!
    ssao_bindings: BindGroup,
    blur_bindings: BindGroup,
    lighting_bindings: BindGroup,
    fxaa_bindings: BindGroup,
    composite_bindings: BindGroup,
}

impl ModelPipeline {
    pub fn new(device: &Device, texture: TextureFormat) -> Self {
        let sampler = device.create_sampler(&NONFILTERING_SAMPLER);
        let filtering_sampler = device.create_sampler(&FILTERING_SAMPLER);

        let post_index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[0, 1, 2]),
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
        });

        Self {
            viewport: None,
            preview: None,
            picker: ModelPicker::new(device),
            base: BasePass::create(device, texture),
            ssao: SsaoPass::create(device),
            blur: BlurPass::create(device),
            lighting: LightingPass::create(device, texture),
            fxaa: FxaaPass::create(device, texture),
            composite: CompositePass::create(device, texture),

            post_index_buffer,
            sampler,
            filtering_sampler,
        }
    }

    fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        app: &mut App,
        target: Target,
        background: Option<SRgb<f32>>,
    ) {
        let multi = self.get_target(target).as_ref().unwrap();
        let index = &self.post_index_buffer;

        self.base.paint(encoder, multi, app);
        if app.config.render.ambient_occlusion.enabled {
            self.ssao.paint(encoder, multi, index, &multi.ssao_bindings);
            self.blur.paint(encoder, multi, index, &multi.blur_bindings);
        }

        self.lighting
            .paint(encoder, multi, index, background, &multi.lighting_bindings);
        if app.config.render.anti_aliasing.enabled {
            self.fxaa.paint(encoder, multi, index, &multi.fxaa_bindings);
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
        let occlusion_size = occlusion_size(app, size);

        self.base.prepare(gcx, app);
        self.ssao.prepare(gcx, app, None);
        self.blur.prepare(gcx, app, occlusion_size);
        self.lighting.prepare(gcx, app, None);
        self.fxaa.prepare(gcx, app, size);

        self.size_textures(gcx, app, Target::Viewport, screen.size_in_pixels.into());
        self.render(encoder, app, Target::Viewport, None);

        let multi = self.viewport.as_ref().unwrap();
        self.picker.update(gcx, encoder, multi, app);
    }

    // Runs the post-processing pipeline, copying the data from the intermediary
    // buffers to the output surface.
    pub fn paint(&self, render_pass: &mut RenderPass, _app: &mut App) {
        let bindings = &self.viewport.as_ref().unwrap().composite_bindings;
        self.composite
            .paint(render_pass, &self.post_index_buffer, bindings);
    }

    fn get_target_mut(&mut self, target: Target) -> &mut Option<MultiStage> {
        match target {
            Target::Viewport => &mut self.viewport,
            Target::Preview => &mut self.preview,
        }
    }

    fn get_target(&self, target: Target) -> &Option<MultiStage> {
        match target {
            Target::Viewport => &self.viewport,
            Target::Preview => &self.preview,
        }
    }
}
