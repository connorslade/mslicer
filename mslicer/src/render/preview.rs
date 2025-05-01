use std::f32::consts::PI;

use image::{Rgba, RgbaImage};
use nalgebra::{Matrix4, Vector2, Vector3};
use tracing::{error, info};
use wgpu::{
    BufferAddress, BufferDescriptor, BufferUsages, Color, CommandEncoderDescriptor, Device,
    Extent3d, ImageCopyBuffer, ImageCopyTexture, ImageDataLayout, LoadOp, Maintain, MapMode,
    Operations, Origin3d, Queue, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, StoreOp, Texture, TextureAspect, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};

use crate::{app::App, DEPTH_TEXTURE_FORMAT};

use super::{
    camera::Camera,
    pipelines::model::ModelPipeline,
    workspace::{WorkspaceRenderCallback, WorkspaceRenderResources},
};

pub fn process_previews(app: &App) {
    match &app.slice_operation {
        Some(slice_operation) if slice_operation.needs_preview_image() => {
            let image = render_preview_image(app, (512, 512));
            slice_operation.add_preview_image(image);
        }
        _ => {}
    }
}

// TODO: Allow rendering multiple preview images at once
fn render_preview_image(app: &App, size: (u32, u32)) -> RgbaImage {
    info!("Generating {}x{} preview image", size.0, size.1);

    let mut resources = app.get_callback_resource_mut::<WorkspaceRenderResources>();
    let (device, queue) = (&app.render_state.device, &app.render_state.queue);

    let mut workspace = app.get_workspace_render_callback(Matrix4::zeros(), false);

    let format = app.render_state.target_format;
    let (texture, resolved_texture, depth_texture) = init_textures(device, format, size);
    let texture_view = texture.create_view(&TextureViewDescriptor::default());
    let resolved_texture_view = resolved_texture.create_view(&TextureViewDescriptor::default());
    let depth_texture_view = depth_texture.create_view(&TextureViewDescriptor::default());

    let (mut min, mut max) = (Vector3::repeat(f32::MAX), Vector3::repeat(f32::MIN));
    for model in workspace.models.read().iter() {
        let (model_min, model_max) = model.mesh.bounds();
        min = min.zip_map(&model_min, f32::min);
        max = max.zip_map(&model_max, f32::max);
    }

    let target = (min + max) / 2.0;
    let distance = (min - max).magnitude() / 2.0;

    workspace.camera = Camera {
        target,
        distance,
        angle: Vector2::new(0.0, PI / 10.0),
        ..workspace.camera
    };

    let aspect = size.0 as f32 / size.1 as f32;
    workspace.transform = workspace.camera.view_projection_matrix(aspect);

    resources.model_pipeline.prepare(device, &workspace);

    render_preview(
        device,
        queue,
        &resources.model_pipeline,
        &workspace,
        &texture_view,
        &resolved_texture_view,
        &depth_texture_view,
    );

    download_preview(device, format, queue, &resolved_texture)
}

fn render_preview(
    device: &Device,
    queue: &Queue,
    model_pipeline: &ModelPipeline,
    workspace: &WorkspaceRenderCallback,
    texture_view: &TextureView,
    resolved_texture_view: &TextureView,
    depth_texture_view: &TextureView,
) {
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
    let mut preview_render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(RenderPassColorAttachment {
            view: texture_view,
            resolve_target: Some(resolved_texture_view),
            ops: Operations {
                load: LoadOp::Clear(Color::BLACK),
                store: StoreOp::Store,
            },
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

    model_pipeline.paint(&mut preview_render_pass, workspace);
    drop(preview_render_pass);
    queue.submit(std::iter::once(encoder.finish()));
}

fn download_preview(
    device: &Device,
    format: TextureFormat,
    queue: &Queue,
    texture: &Texture,
) -> RgbaImage {
    let mut download_encoder =
        device.create_command_encoder(&CommandEncoderDescriptor { label: None });
    let texture_extent = texture.size();
    let texture_size = (texture_extent.width * texture_extent.height * 4) as BufferAddress;

    let staging_buffer = device.create_buffer(&BufferDescriptor {
        label: None,
        size: texture_size,
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    download_encoder.copy_texture_to_buffer(
        ImageCopyTexture {
            texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        ImageCopyBuffer {
            buffer: &staging_buffer,
            layout: ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * texture_extent.width),
                rows_per_image: Some(texture_extent.height),
            },
        },
        texture_extent,
    );
    queue.submit(std::iter::once(download_encoder.finish()));

    let (tx, rx) = std::sync::mpsc::channel();
    let slice = staging_buffer.slice(..);
    slice.map_async(MapMode::Read, move |_| tx.send(()).unwrap());

    device.poll(Maintain::Wait);
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
            error!("Can't make preview image due to unsupported framebuffer texture format {x:?}. Please make an issue on Github.");
            RgbaImage::new(width, height)
        }
    };

    drop(mapped_range);
    staging_buffer.unmap();

    image
}

fn init_textures(
    device: &Device,
    format: TextureFormat,
    size: (u32, u32),
) -> (Texture, Texture, Texture) {
    let size = Extent3d {
        width: size.0,
        height: size.1,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 4,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    let resolved_texture = device.create_texture(&TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    let depth_texture = device.create_texture(&TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 4,
        dimension: TextureDimension::D2,
        format: DEPTH_TEXTURE_FORMAT,
        usage: TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    (texture, resolved_texture, depth_texture)
}
