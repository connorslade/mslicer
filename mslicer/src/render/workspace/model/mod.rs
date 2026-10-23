use std::ops::Deref;

use common::color::SRgb;
use nalgebra::{Matrix4, Vector2};
use rand::random_range;
use wgpu::{
    Buffer, BufferUsages, CommandEncoder, Device, Origin3d, RenderPass, Sampler,
    TexelCopyTextureInfo, TextureAspect, TextureFormat, TextureView,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    core::App,
    render::{
        Gcx,
        consts::{FILTERING_SAMPLER, NONFILTERING_SAMPLER},
        workspace::model::{
            bindings::texture_size,
            pass::{
                base::{BasePass, BaseResources},
                blur::{BlurPass, BlurResources},
                composite::{CompositePass, CompositeResources},
                fxaa::{FxaaPass, FxaaResources},
                lighting::{LightingPass, LightingResources},
                ssao::{SsaoPass, SsaoResources},
            },
            pick::ModelPicker,
        },
    },
};

mod bindings;
mod pass;
pub mod pick;
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

pub struct MultiStage {
    textures: TextureViews,

    //bind groups!
    base: BaseResources,
    ssao: SsaoResources,
    blur: BlurResources,
    lighting: LightingResources,
    fxaa: FxaaResources,
    composite: CompositeResources,
}

#[derive(Clone)]
pub struct TextureViews {
    pub target_a: TextureView,
    pub target_b: TextureView,
    pub accumulation: Buffer,

    pub occlusion_target_a: TextureView,
    pub occlusion_target_b: TextureView,

    pub depth_target: TextureView,
    pub model_target: TextureView,
    pub normal_target: TextureView,
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
        [ssao, fxaa]: [bool; 2],
        background: Option<SRgb<f32>>,
    ) {
        let multi = self.get_target(target).as_ref().unwrap();
        let index = &self.post_index_buffer;

        match target {
            Target::Viewport => self.base.paint(encoder, app, multi),
            Target::Preview => self.base.paint_preview(encoder, app, multi),
        }
        if ssao {
            self.ssao.paint(encoder, multi, index);
            self.blur.paint(encoder, multi, index);
        }

        self.lighting.paint(encoder, multi, index, background);
        if fxaa {
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
        encoder: &mut CommandEncoder,
        app: &mut App,
        size: Vector2<u32>,
    ) {
        let occlusion_size = texture_size(app.config.render.ambient_occlusion.scale, size);
        let mut view_projection = app.view_projection();

        let render = &app.config.render;
        let taa = render.anti_aliasing.taa();
        let passes @ [ao, _] = [
            render.ambient_occlusion.enabled,
            render.anti_aliasing.fxaa(),
        ];

        if taa && !app.state.workspace.is_moving {
            let jitter = Vector2::<f32>::new(random_range(-0.5..=0.5), random_range(-0.5..=0.5));
            let clip = jitter.component_div(&size.cast::<f32>()) * 2.0;
            view_projection = Matrix4::new_translation(&clip.to_homogeneous()) * view_projection;
        }

        self.size_textures(gcx, app, Target::Viewport, size);
        let multi = self.viewport.as_mut().unwrap();
        multi.base.prepare(gcx, app, &self.base, view_projection);

        multi.ssao.prepare(gcx, app, view_projection);
        multi.blur.prepare(gcx, app, occlusion_size);
        multi.lighting.prepare(gcx, app, ao, None);
        multi.fxaa.prepare(gcx, app, size);
        multi.composite.prepare(gcx, app, taa, size);
        self.render(encoder, app, Target::Viewport, passes, None);

        let multi = self.viewport.as_ref().unwrap();
        self.picker.update(app, gcx, encoder, multi);
    }

    pub fn prepare_preview(
        &mut self,
        gcx: &Gcx,
        encoder: &mut CommandEncoder,
        app: &mut App,
        size: Vector2<u32>,
    ) {
        let Some(Some(interactive)) =
            (app.slice_operation.as_ref()).map(|x| x.interactive_previews.as_ref())
        else {
            return;
        };

        let occlusion_size = texture_size(app.config.render.ambient_occlusion.scale, size);
        let aspect = size.x as f32 / size.y as f32;
        let camera = interactive.camera.lock().clone();

        let preview = &app.config.render.preview;
        let background = preview.background;
        let view = camera.view_projection_matrix(preview.projection, aspect);

        self.size_textures(gcx, app, Target::Preview, size);
        let multi = self.preview.as_mut().unwrap();
        multi.base.prepare_preview(gcx, app, &self.base, view);

        multi.ssao.prepare(gcx, app, view);
        multi.blur.prepare(gcx, app, occlusion_size);
        multi.lighting.prepare(gcx, app, true, Some(&camera));
        multi.fxaa.prepare(gcx, app, size);
        multi.composite.prepare(gcx, app, false, size);
        self.render(encoder, app, Target::Preview, [true; 2], Some(background));
    }

    // Runs the post-processing pipeline, copying the data from the intermediary
    // buffers to the output surface.
    pub fn paint(&self, render_pass: &mut RenderPass, _app: &mut App, target: Target) {
        let resources = &self.get_target(target).as_ref().unwrap().composite;
        self.composite
            .paint(render_pass, &self.post_index_buffer, resources);
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

impl Deref for MultiStage {
    type Target = TextureViews;

    fn deref(&self) -> &Self::Target {
        &self.textures
    }
}
