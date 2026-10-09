use egui_wgpu::RenderState;
use image::{Rgba, RgbaImage, imageops};
use nalgebra::Vector2;
use parking_lot::MappedRwLockWriteGuard;
use tracing::{error, info};
use wgpu::{
    BufferAddress, BufferDescriptor, BufferUsages, COPY_BYTES_PER_ROW_ALIGNMENT, Extent3d, MapMode,
    Origin3d, PollType, TexelCopyBufferInfo, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture,
    TextureAspect, TextureFormat,
};

use crate::{
    core::App,
    render::{
        Gcx,
        workspace::{WorkspaceRenderResources, model::ModelPipeline},
    },
};

pub fn process_previews(app: &mut App) {
    if let Some(operation) = &app.slice_operation
        && operation.needs_previews()
    {
        // yes i know im downloading a texture from the gpu and then immediately
        // reuploading it... sue me.
        let preview = &app.config.render.preview;
        let image = render_preview_image(app, preview.size);
        let operation = app.slice_operation.as_ref().unwrap();
        operation.add_preview(image);
    }
}

// TODO: Allow rendering multiple preview images at once
fn render_preview_image(app: &mut App, size: Vector2<u32>) -> RgbaImage {
    info!("Generating {}x{} preview image", size.x, size.y);

    let gcx = app.gcx();
    let mut encoder = gcx.device.create_command_encoder(&Default::default());

    let render_state = app.render_state.clone();
    let mut pipeline = pipeline(&render_state);

    pipeline.prepare_preview(&gcx, &mut encoder, app, size);
    let texture = pipeline.preview.as_ref().unwrap().target_a.clone();

    gcx.queue.submit(std::iter::once(encoder.finish()));
    download_preview(&gcx, texture.texture(), size)
}

fn download_preview(gcx: &Gcx, texture: &Texture, size: Vector2<u32>) -> RgbaImage {
    let width = size.x.next_multiple_of(COPY_BYTES_PER_ROW_ALIGNMENT);

    let mut download_encoder = gcx.device.create_command_encoder(&Default::default());
    let texture_extent = texture.size();
    let texture_size = (width * texture_extent.height * 4) as BufferAddress;

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
                bytes_per_row: Some(4 * width),
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
    let Extent3d { height, .. } = texture_extent;
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
            imageops::crop_imm(&image, 0, 0, size.x, height).to_image()
        }
        x => {
            error!(
                "Can't make preview image due to unsupported framebuffer texture format {x:?}. Please make an issue on Github."
            );
            RgbaImage::new(size.x, height)
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
