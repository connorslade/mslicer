// TODO: Will need to be updated again to run the post processing shader.

use std::f32::consts::PI;
use std::mem;

use egui_wgpu::RenderState;
use encase::UniformBuffer;
use image::{Rgba, RgbaImage};
use nalgebra::{Vector2, Vector3};
use parking_lot::MappedRwLockWriteGuard;
use tracing::{error, info};
use wgpu::{
    BufferAddress, BufferDescriptor, BufferUsages, CommandEncoder, Extent3d, MapMode, Origin3d,
    PollType, TexelCopyBufferInfo, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture,
    TextureAspect, TextureFormat, TextureView,
};

use crate::app::App;
use crate::app::config::RenderStyle;
use crate::render::camera::Projection;
use crate::render::workspace::model::{ModelUniforms, PostUniforms};
use crate::render::{
    Gcx,
    camera::Camera,
    workspace::{WorkspaceRenderResources, model::ModelPipeline},
};

impl ModelPipeline {
    // Similar to `Self::prepare`, but render target buffers are only held for
    // the single operation.
    fn render_preview(
        &mut self,
        gcx: &Gcx,
        encoder: &mut CommandEncoder,
        app: &mut App,
        size: Vector2<u32>,
        camera: Camera,
    ) -> TextureView {
        self.upload_preview_uniforms(gcx, app, &camera);

        // Switch out the multi stage state just for this operation
        let mut old = self.multi_stage.take();
        self.size_textures(gcx, size);
        self.render(encoder, app);
        mem::swap(&mut self.multi_stage, &mut old);

        old.unwrap().resolved_target
    }

    fn upload_preview_uniforms(&mut self, gcx: &Gcx, app: &mut App, camera: &Camera) {
        let view_projection = camera.view_projection_matrix(Projection::Perspective, 1.0);
        self.bind_groups.clear();
        for model in app.project.models.iter_mut() {
            model.get_buffers(&gcx.device);

            let model_transform = *model.mesh.transformation_matrix();
            let uniforms = ModelUniforms {
                transform: view_projection * model_transform,
                model_transform,
                build_volume: Vector3::repeat(f32::MAX),
                model_color: model.color.to_srgb().into(),
                camera_position: camera.position(camera.distance),
                render_style: RenderStyle::Rendered as u32,
                overhang_angle: 0.0,
            };
            self.bind_groups.push(self.bind_group(gcx, uniforms));
        }

        let post_uniform = PostUniforms {
            view: view_projection,
            inv_view: view_projection.try_inverse().unwrap(),
        };

        let mut buffer = UniformBuffer::new(Vec::new());
        buffer.write(&post_uniform).unwrap();
        gcx.queue
            .write_buffer(&self.post_uniform, 0, &buffer.into_inner());
    }
}

pub fn process_previews(app: &mut App) {
    match &app.slice_operation {
        Some(slice_operation) if slice_operation.needs_preview_image() => {
            let image = render_preview_image(app, Vector2::repeat(512));
            (app.slice_operation.as_ref().unwrap()).add_preview_image(image);
        }
        _ => {}
    }
}

// TODO: Allow rendering multiple preview images at once
fn render_preview_image(app: &mut App, size: Vector2<u32>) -> RgbaImage {
    info!("Generating {}x{} preview image", size.x, size.y);
    let gcx = app.gcx();

    let (mut min, mut max) = (Vector3::repeat(f32::MAX), Vector3::repeat(f32::MIN));
    for model in app.project.models.iter() {
        let (model_min, model_max) = model.mesh.bounds();
        min = min.zip_map(&model_min, f32::min);
        max = max.zip_map(&model_max, f32::max);
    }

    let mut camera = Camera {
        target: (min + max) / 2.0,
        ..Default::default()
    };
    camera.angle.y = PI / 10.0;
    camera.distance = (max - camera.target).magnitude() / (camera.fov / 2.0).tan();

    let render_state = app.render_state.clone();

    let mut encoder = gcx.device.create_command_encoder(&Default::default());
    let texture = pipeline(&render_state).render_preview(&gcx, &mut encoder, app, size, camera);
    gcx.queue.submit(std::iter::once(encoder.finish()));

    download_preview(&gcx, texture.texture())
}

fn download_preview(gcx: &Gcx, texture: &Texture) -> RgbaImage {
    let mut download_encoder = gcx.device.create_command_encoder(&Default::default());
    let texture_extent = texture.size();
    let texture_size = (texture_extent.width * texture_extent.height * 4) as BufferAddress;

    let staging_buffer = gcx.device.create_buffer(&BufferDescriptor {
        label: None,
        size: texture_size,
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    download_encoder.copy_texture_to_buffer(
        TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        TexelCopyBufferInfo {
            buffer: &staging_buffer,
            layout: TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * texture_extent.width),
                rows_per_image: Some(texture_extent.height),
            },
        },
        texture_extent,
    );
    gcx.queue.submit(std::iter::once(download_encoder.finish()));

    let (tx, rx) = std::sync::mpsc::channel();
    let slice = staging_buffer.slice(..);
    slice.map_async(MapMode::Read, move |_| tx.send(()).unwrap());

    gcx.device.poll(PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap();

    let mapped_range = slice.get_mapped_range();
    let result = bytemuck::cast_slice::<_, u8>(&mapped_range);

    // Convert texture to to RGBA image. Format is *not* guaranteed to be be,
    // but will almost always be Rgba8Unorm or Bgra8Unorm.
    let Extent3d { width, height, .. } = texture_extent;
    let image = match gcx.texture {
        TextureFormat::Rgba8Unorm => RgbaImage::from_raw(width, height, result.to_vec()).unwrap(),
        TextureFormat::Bgra8Unorm => {
            let mut image = RgbaImage::from_raw(width, height, result.to_vec()).unwrap();
            for y in 0..image.height() {
                for x in 0..image.width() {
                    let bgra = image.get_pixel(x, y).0;
                    image.put_pixel(x, y, Rgba([bgra[2], bgra[1], bgra[0], bgra[3]]));
                }
            }
            image
        }
        x => {
            error!(
                "Can't make preview image due to unsupported framebuffer texture format {x:?}. Please make an issue on Github."
            );
            RgbaImage::new(width, height)
        }
    };

    drop(mapped_range);
    staging_buffer.unmap();

    image
}

fn pipeline(render_state: &RenderState) -> MappedRwLockWriteGuard<'_, ModelPipeline> {
    MappedRwLockWriteGuard::map(render_state.renderer.write(), |x| {
        &mut (x.callback_resources)
            .get_mut::<WorkspaceRenderResources>()
            .unwrap()
            .model
    })
}
