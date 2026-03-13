use std::f32::consts::PI;

use egui_wgpu::RenderState;
use image::{Rgba, RgbaImage};
use nalgebra::Vector3;
use parking_lot::MappedRwLockWriteGuard;
use tracing::{error, info};
use wgpu::{
    BufferAddress, BufferDescriptor, BufferUsages, Color, Extent3d, LoadOp, MapMode, Operations,
    Origin3d, PollType, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, StoreOp, TexelCopyBufferInfo, TexelCopyBufferLayout,
    TexelCopyTextureInfo, Texture, TextureAspect, TextureFormat, TextureView,
};

use crate::app::App;
use crate::render::{
    Gcx,
    camera::Camera,
    util::init_textures,
    workspace::{WorkspaceRenderResources, model::ModelPipeline},
};

pub fn process_previews(app: &mut App) {
    match &app.slice_operation {
        Some(slice_operation) if slice_operation.needs_preview_image() => {
            let image = render_preview_image(app, (512, 512));
            (app.slice_operation.as_ref().unwrap()).add_preview_image(image);
        }
        _ => {}
    }
}

// TODO: Allow rendering multiple preview images at once
fn render_preview_image(app: &mut App, size: (u32, u32)) -> RgbaImage {
    info!("Generating {}x{} preview image", size.0, size.1);
    let gcx = app.gcx();

    let format = app.render_state.target_format;
    let (texture, resolved_texture, depth_texture) = init_textures(&gcx.device, format, size);
    let texture_view = texture.create_view(&Default::default());
    let resolved_texture_view = resolved_texture.create_view(&Default::default());
    let depth_texture_view = depth_texture.create_view(&Default::default());

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
    render_preview(
        app,
        &gcx,
        &mut pipeline(&render_state),
        &texture_view,
        &resolved_texture_view,
        &depth_texture_view,
        camera,
    );

    download_preview(&gcx, format, &resolved_texture)
}

fn render_preview(
    app: &mut App,
    gcx: &Gcx,
    model_pipeline: &mut ModelPipeline,
    texture_view: &TextureView,
    resolved_texture_view: &TextureView,
    depth_texture_view: &TextureView,
    camera: Camera,
) {
    let mut encoder = gcx.device.create_command_encoder(&Default::default());
    let mut preview_render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(RenderPassColorAttachment {
            view: texture_view,
            resolve_target: Some(resolved_texture_view),
            ops: Operations {
                load: LoadOp::Clear(Color::BLACK),
                store: StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
            view: depth_texture_view,
            depth_ops: Some(Operations {
                load: LoadOp::Clear(1.0),
                store: StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
    });

    model_pipeline.prepare_preview(gcx, app, camera);
    model_pipeline.paint(&mut preview_render_pass, app);
    drop(preview_render_pass);
    gcx.queue.submit(std::iter::once(encoder.finish()));
}

fn download_preview(gcx: &Gcx, format: TextureFormat, texture: &Texture) -> RgbaImage {
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
    let image = match format {
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
