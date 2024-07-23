use image::RgbaImage;
use wgpu::{
    Color, CommandEncoderDescriptor, Device, LoadOp, Queue, RenderPassDescriptor, StoreOp, Texture,
    TextureDescriptor, TextureDimension, TextureUsages, TextureView, TextureViewDescriptor,
};

use crate::TEXTURE_FORMAT;

use super::{
    pipelines::{model::ModelPipeline, Pipeline},
    workspace::WorkspaceRenderCallback,
};

// TODO: Allow rendering multiple preview images at once
pub fn render_preview_image(
    device: &Device,
    queue: &Queue,
    size: (u32, u32),
    model_pipeline: &ModelPipeline,
    workspace: &WorkspaceRenderCallback,
) -> RgbaImage {
    let (texture, depth_texture) = init_textures(device, size);
    let texture_view = texture.create_view(&TextureViewDescriptor::default());
    let depth_texture_view = depth_texture.create_view(&TextureViewDescriptor::default());

    render_preview(
        device,
        queue,
        model_pipeline,
        workspace,
        &texture_view,
        &depth_texture_view,
    );

    download_preview(device, queue, &texture)
}

fn render_preview(
    device: &Device,
    queue: &Queue,
    model_pipeline: &ModelPipeline,
    workspace: &WorkspaceRenderCallback,
    texture_view: &TextureView,
    depth_texture_view: &TextureView,
) {
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
    let mut preview_render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &texture_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: LoadOp::Clear(Color::BLACK),
                store: StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &depth_texture_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
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

fn download_preview(device: &Device, queue: &Queue, texture: &Texture) -> RgbaImage {
    let mut download_encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    let texture_extent = texture.size();
    let texture_size = (texture_extent.width * texture_extent.height * 4) as wgpu::BufferAddress;
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: texture_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    download_encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
            buffer: &staging_buffer,
            layout: wgpu::ImageDataLayout {
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
    slice.map_async(wgpu::MapMode::Read, move |_| tx.send(()).unwrap());

    device.poll(wgpu::Maintain::Wait);
    rx.recv().unwrap();

    let mapped_range = slice.get_mapped_range();
    let result = bytemuck::cast_slice::<_, u8>(&mapped_range);

    let image =
        image::RgbaImage::from_raw(texture_extent.width, texture_extent.height, result.to_vec())
            .unwrap();

    drop(mapped_range);
    staging_buffer.unmap();

    image
}

fn init_textures(device: &Device, size: (u32, u32)) -> (Texture, Texture) {
    let texture = device.create_texture(&TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TEXTURE_FORMAT,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    let depth_texture = device.create_texture(&TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: 1024,
            height: 1024,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth24PlusStencil8,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    (texture, depth_texture)
}
