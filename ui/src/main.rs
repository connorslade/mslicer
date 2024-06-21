use std::{fs::File, mem};

use anyhow::Result;
use eframe::NativeOptions;
use egui::Vec2;
use render::ModelVertex;
use wgpu::{
    BindGroupEntry, BindGroupLayoutDescriptor, BufferAddress, BufferBinding, BufferDescriptor,
    BufferUsages, ColorTargetState, ColorWrites, CompareFunction, DepthStencilState, FragmentState,
    MultisampleState, PipelineLayoutDescriptor, PrimitiveState, RenderPipelineDescriptor,
    ShaderModuleDescriptor, TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexState, VertexStepMode,
};

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Bgra8Unorm;

mod app;
mod camera;
mod render;
mod workspace;
use app::App;
use workspace::{WorkspaceRenderCallback, WorkspaceRenderResources};

fn main() -> Result<()> {
    eframe::run_native(
        "mslicer",
        NativeOptions {
            window_builder: Some(Box::new(|builder| {
                builder.with_inner_size(Vec2::new(1920.0, 1080.0))
            })),

            depth_buffer: 24,
            stencil_buffer: 8,
            ..Default::default()
        },
        // todo: move to some other file or somthin
        Box::new(|cc| {
            let render_state = cc.wgpu_render_state.as_ref().unwrap();
            let device = &render_state.device;

            let shader = device.create_shader_module(ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("shader/workspace.wgsl").into()),
            });

            let uniform_buffer = device.create_buffer(&BufferDescriptor {
                label: None,
                size: mem::size_of::<WorkspaceRenderCallback>() as u64,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

            let vertex_buffers = [VertexBufferLayout {
                array_stride: mem::size_of::<ModelVertex>() as BufferAddress,
                step_mode: VertexStepMode::Vertex,
                attributes: &[
                    VertexAttribute {
                        format: VertexFormat::Float32x4,
                        offset: 0,
                        shader_location: 0,
                    },
                    VertexAttribute {
                        format: VertexFormat::Float32x2,
                        offset: 4 * 4,
                        shader_location: 1,
                    },
                    VertexAttribute {
                        format: VertexFormat::Float32x3,
                        offset: 4 * 4 + 4 * 2,
                        shader_location: 2,
                    },
                ],
            }];

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
                    buffers: &vertex_buffers,
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
                primitive: PrimitiveState {
                    ..Default::default()
                },
                depth_stencil: Some(DepthStencilState {
                    format: TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: MultisampleState::default(),
                multiview: None,
            });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                }],
            });

            render_state
                .renderer
                .write()
                .callback_resources
                .insert(WorkspaceRenderResources {
                    uniform_buffer,

                    render_pipeline,
                    bind_group,
                });

            Box::new(App::default())
        }),
    )
    .unwrap();

    Ok(())
}
