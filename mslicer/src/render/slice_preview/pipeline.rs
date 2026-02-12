use encase::{ShaderSize, ShaderType, UniformBuffer};
use nalgebra::Vector2;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Buffer, BufferDescriptor, BufferUsages, COPY_BUFFER_ALIGNMENT,
    ColorTargetState, ColorWrites, Device, FragmentState, IndexFormat, MultisampleState,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, TextureFormat, VertexState,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    include_shader,
    render::{
        VERTEX_BUFFER_LAYOUT,
        consts::{
            BASE_UNIFORM_DESCRIPTOR, DEPTH_STENCIL_STATE, STORAGE_BIND_GROUP_LAYOUT_ENTRY,
            UNIFORM_BIND_GROUP_LAYOUT_ENTRY,
        },
        slice_preview::SlicePreviewRenderCallback,
    },
};

pub struct SlicePreviewPipeline {
    render_pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,
    bind_group: Option<BindGroup>,

    index_buffer: Buffer,
    uniform_buffer: Buffer,
    slice_buffer: Option<SliceBuffers>,
}

struct SliceBuffers {
    layer: Buffer,
    annotations: Buffer,
}

#[derive(ShaderType)]
struct SlicePreviewUniforms {
    dimensions: Vector2<u32>,
    offset: Vector2<f32>,
    aspect: f32,
    scale: f32,
}

impl SlicePreviewPipeline {
    pub fn new(device: &Device, texture: TextureFormat) -> Self {
        let shader = device.create_shader_module(include_shader!("slice_preview.wgsl"));

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

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
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
            render_pipeline,
            bind_group_layout,
            bind_group: None,

            index_buffer,
            uniform_buffer,

            slice_buffer: None,
        }
    }
}

impl SlicePreviewPipeline {
    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        resources: &SlicePreviewRenderCallback,
    ) {
        let slice_buffer = self.slice_buffer.get_or_insert_with(|| {
            let size = resources.dimensions.x as u64 * resources.dimensions.y as u64;
            let desc = BufferDescriptor {
                label: None,
                size: size.next_multiple_of(COPY_BUFFER_ALIGNMENT),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            };

            SliceBuffers {
                layer: device.create_buffer(&desc),
                annotations: device.create_buffer(&BufferDescriptor {
                    // size: size.div_ceil(2).next_multiple_of(COPY_BUFFER_ALIGNMENT),
                    ..desc
                }),
            }
        });

        if let Some((layer, annotations)) = &resources.new_preview {
            queue.write_buffer(&slice_buffer.layer, 0, layer);
            queue.write_buffer(&slice_buffer.annotations, 0, annotations);
        }

        self.bind_group = Some(device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: slice_buffer.layer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: slice_buffer.annotations.as_entire_binding(),
                },
            ],
        }));

        let mut buffer = UniformBuffer::new(Vec::new());
        buffer
            .write(&SlicePreviewUniforms {
                dimensions: resources.dimensions,
                offset: resources.offset,
                aspect: resources.aspect,
                scale: resources.scale.recip(),
            })
            .unwrap();
        queue.write_buffer(&self.uniform_buffer, 0, &buffer.into_inner());
    }

    pub fn paint(&self, render_pass: &mut RenderPass) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);

        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..3, 0, 0..1);
    }
}
