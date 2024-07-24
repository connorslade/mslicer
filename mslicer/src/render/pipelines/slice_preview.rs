use egui_wgpu::ScreenDescriptor;
use encase::{ShaderSize, ShaderType, UniformBuffer};
use nalgebra::Vector2;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding, BufferBindingType,
    BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, CommandEncoder, CompareFunction,
    DepthStencilState, Device, FragmentState, IndexFormat, MultisampleState,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, TextureFormat,
    VertexState,
};

use crate::{
    include_shader,
    render::{slice_preview::SlicePreviewRenderCallback, ModelVertex, VERTEX_BUFFER_LAYOUT},
    TEXTURE_FORMAT,
};

use super::{
    consts::{BASE_UNIFORM_DESCRIPTOR, UNIFORM_BIND_GROUP_LAYOUT_ENTRY},
    Pipeline,
};

pub struct SlicePreviewPipeline {
    render_pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,
    bind_group: Option<BindGroup>,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffer: Buffer,
    slice_buffer: Option<Buffer>,
}

#[derive(ShaderType)]
struct SlicePreviewUniforms {
    dimensions: Vector2<u32>,
    offset: Vector2<f32>,
    scale: f32,
}

impl SlicePreviewPipeline {
    pub fn new(device: &Device) -> Self {
        let shader = device.create_shader_module(include_shader!("slice_preview.wgsl"));

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            size: SlicePreviewUniforms::SHADER_SIZE.get(),
            ..BASE_UNIFORM_DESCRIPTOR
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                UNIFORM_BIND_GROUP_LAYOUT_ENTRY,
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
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
                entry_point: "vert",
                buffers: &[VERTEX_BUFFER_LAYOUT],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "frag",
                targets: &[Some(ColorTargetState {
                    format: TEXTURE_FORMAT,
                    blend: None,
                    write_mask: ColorWrites::all(),
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: MultisampleState {
                count: 4,
                ..Default::default()
            },
            multiview: None,
        });

        let vert = [[-1.0, -1.0], [3.0, -1.0], [-1.0, 3.0]]
            .into_iter()
            .map(|[x, y]| ModelVertex {
                position: [x, y, 0.0, 1.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            })
            .collect::<Vec<_>>();
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Platform Vertex Buffer"),
            contents: bytemuck::cast_slice(&vert),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Platform Index Buffer"),
            contents: bytemuck::cast_slice(&[0, 1, 2]),
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
        });

        Self {
            render_pipeline,
            bind_group_layout,
            bind_group: None,

            vertex_buffer,
            index_buffer,
            uniform_buffer,

            slice_buffer: None,
        }
    }
}

impl Pipeline<SlicePreviewRenderCallback> for SlicePreviewPipeline {
    fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        _screen_descriptor: &ScreenDescriptor,
        _encoder: &mut CommandEncoder,
        resources: &SlicePreviewRenderCallback,
    ) {
        let slice_buffer = self.slice_buffer.take().unwrap_or_else(|| {
            device.create_buffer(&BufferDescriptor {
                label: None,
                size: resources.dimensions.x as u64 * resources.dimensions.y as u64,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });
        self.slice_buffer = Some(slice_buffer);

        if let Some(new_preview) = &resources.new_preview {
            queue.write_buffer(self.slice_buffer.as_ref().unwrap(), 0, new_preview);
        }

        self.bind_group = Some(device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &self.uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: self.slice_buffer.as_ref().unwrap(),
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        }));

        let mut buffer = UniformBuffer::new(Vec::new());
        buffer
            .write(&SlicePreviewUniforms {
                dimensions: resources.dimensions,
                offset: resources.offset,
                scale: resources.scale.recip(),
            })
            .unwrap();
        queue.write_buffer(&self.uniform_buffer, 0, &buffer.into_inner());
    }

    fn paint<'a>(
        &'a self,
        render_pass: &mut RenderPass<'a>,
        _resources: &SlicePreviewRenderCallback,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..3, 0, 0..1);
    }
}
