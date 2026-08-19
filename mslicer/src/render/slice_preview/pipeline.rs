use encase::{ShaderSize, ShaderType, UniformBuffer};
use nalgebra::Vector2;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Buffer, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites,
    CommandEncoder, Device, FragmentState, IndexFormat, MultisampleState, PipelineLayoutDescriptor,
    PrimitiveState, RenderPass, RenderPipeline, RenderPipelineDescriptor, TextureFormat,
    VertexState,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    include_shader,
    render::{
        Gcx, VERTEX_BUFFER_LAYOUT,
        consts::{
            BASE_UNIFORM_DESCRIPTOR, DEPTH_STENCIL_STATE, STORAGE_BIND_GROUP_LAYOUT_ENTRY,
            UNIFORM_BIND_GROUP_LAYOUT_ENTRY,
        },
        slice_preview::{
            SlicePreviewRenderCallback,
            decompress::{DecompressPass, DecompressedBuffer},
        },
    },
};

pub struct SlicePreviewPipeline {
    pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,
    bind_group: Option<BindGroup>,

    index_buffer: Buffer,
    uniform_buffer: Buffer,

    decompress: DecompressPass,
    layer: DecompressedBuffer,
    annotations: DecompressedBuffer,
}

#[derive(ShaderType)]
struct SlicePreviewUniforms {
    dimensions: Vector2<u32>,
    offset: Vector2<f32>,
    scale: Vector2<f32>,
    aspect: f32,
    pixel_aspect: f32,
    multisample: u32,
}

impl SlicePreviewPipeline {
    pub fn new(device: &Device, texture: TextureFormat) -> Self {
        let shader =
            device.create_shader_module(include_shader!("slice_preview.wgsl", "common.wgsl"));

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            size: SlicePreviewUniforms::SHADER_SIZE.get(),
            ..BASE_UNIFORM_DESCRIPTOR
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                UNIFORM_BIND_GROUP_LAYOUT_ENTRY,
                STORAGE_BIND_GROUP_LAYOUT_ENTRY,
                BindGroupLayoutEntry {
                    binding: 2,
                    ..STORAGE_BIND_GROUP_LAYOUT_ENTRY
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: None,
                buffers: &[VERTEX_BUFFER_LAYOUT],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: None,
                targets: &[Some(ColorTargetState {
                    format: texture,
                    blend: None,
                    write_mask: ColorWrites::all(),
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: Some(DEPTH_STENCIL_STATE),
            multisample: MultisampleState {
                count: 4,
                ..Default::default()
            },
            multiview: None,
            cache: None,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[0, 1, 2]),
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
        });

        Self {
            pipeline,
            bind_group_layout,
            bind_group: None,

            index_buffer,
            uniform_buffer,

            decompress: DecompressPass::new(device),
            layer: DecompressedBuffer::new(device),
            annotations: DecompressedBuffer::new(device),
        }
    }
}

impl SlicePreviewPipeline {
    pub fn prepare(
        &mut self,
        gcx: &Gcx,
        encoder: &mut CommandEncoder,
        resources: &SlicePreviewRenderCallback,
    ) {
        if let Some((layer, annotations)) = &resources.new_preview {
            (self.decompress).decompress(gcx, encoder, &mut self.layer, layer);
            (self.decompress).decompress(gcx, encoder, &mut self.annotations, annotations);

            self.bind_group = Some(gcx.device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &self.bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: self.layer.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: self.annotations.as_entire_binding(),
                    },
                ],
            }));
        }

        let mut buffer = UniformBuffer::new(Vec::new());
        buffer
            .write(&SlicePreviewUniforms {
                dimensions: resources.dimensions,
                pixel_aspect: resources.pixel_aspect,
                scale: resources.scale,
                offset: resources.offset,
                aspect: resources.aspect,
                multisample: resources.multisample,
            })
            .unwrap();
        gcx.queue
            .write_buffer(&self.uniform_buffer, 0, &buffer.into_inner());
    }

    pub fn paint(&self, render_pass: &mut RenderPass) {
        if let Some(bind_group) = &self.bind_group {
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);

            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
            render_pass.draw_indexed(0..3, 0, 0..1);
        }
    }
}
