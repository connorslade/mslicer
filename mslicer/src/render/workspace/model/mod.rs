use common::color::SRgb;
use egui_wgpu::ScreenDescriptor;
use nalgebra::Vector2;
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
                base::{BasePass, BaseResources},
                blur::BlurPass,
                composite::CompositePass,
                fxaa::FxaaPass,
                lighting::LightingPass,
                ssao::SsaoPass,
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
    base: BaseResources,
    ssao: BindGroup,
    blur: BindGroup,
    lighting: BindGroup,
    fxaa: BindGroup,
    composite: BindGroup,
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

        match target {
            Target::Viewport => self.base.paint(encoder, app, multi),
            Target::Preview => self.base.paint_preview(encoder, app, multi),
        }
        if app.config.render.ambient_occlusion.enabled {
            self.ssao.paint(encoder, multi, index, &multi.ssao);
            self.blur.paint(encoder, multi, index, &multi.blur);
        }

        self.lighting
            .paint(encoder, multi, index, background, &multi.lighting);
        if app.config.render.anti_aliasing.enabled {
            self.fxaa.paint(encoder, multi, index, &multi.fxaa);
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
        let view_projection = app.view_projection();

        self.size_textures(gcx, app, Target::Viewport, screen.size_in_pixels.into());
        let multi = self.viewport.as_mut().unwrap();
        multi.base.prepare(gcx, app, &self.base);

        self.ssao.prepare(gcx, app, view_projection);
        self.blur.prepare(gcx, app, occlusion_size);
        self.lighting.prepare(gcx, app, None);
        self.fxaa.prepare(gcx, app, size);

        self.render(encoder, app, Target::Viewport, None);

        let multi = self.viewport.as_ref().unwrap();
        self.picker.update(gcx, encoder, multi, app);
    }

    pub fn prepare_preview(
        &mut self,
        gcx: &Gcx,
        encoder: &mut CommandEncoder,
        app: &mut App,
        size: Vector2<u32>,
    ) {
        let aspect = size.x as f32 / size.y as f32;
        let camera = app.state.preview_camera.clone();

        let preview = &app.config.render.preview;
        let background = preview.background_color;
        let view = (app.state.preview_camera).view_projection_matrix(preview.projection, aspect);

        self.size_textures(gcx, app, Target::Preview, size);
        let multi = self.preview.as_mut().unwrap();
        multi.base.prepare_preview(gcx, app, &self.base, view);

        self.ssao.prepare(gcx, app, view);
        self.blur.prepare(gcx, app, occlusion_size(app, size));
        self.lighting.prepare(gcx, app, Some(&camera));
        self.fxaa.prepare(gcx, app, size);
        self.render(encoder, app, Target::Preview, Some(background.to_srgb()));
    }

    // Runs the post-processing pipeline, copying the data from the intermediary
    // buffers to the output surface.
    pub fn paint(&self, render_pass: &mut RenderPass, _app: &mut App, target: Target) {
        let bindings = &self.get_target(target).as_ref().unwrap().composite;
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
