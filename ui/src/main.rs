use std::{fs::File, mem};

use anyhow::Result;
use eframe::NativeOptions;
use egui::{CentralPanel, Frame, Sense, Slider, Stroke, TopBottomPanel, Vec2, Window};
use egui_wgpu::{Callback, CallbackTrait};
use nalgebra::{Matrix4, Point3, Vector3};
use slicer::{mesh::Mesh, Pos};
use wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, Buffer, BufferAddress, BufferBinding,
    BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, FragmentState, IndexFormat,
    MultisampleState, PipelineLayoutDescriptor, PrimitiveState, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, TextureFormat, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Bgra8Unorm;

struct App {
    camera: Camera,
}

struct Camera {
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

struct WorkspaceRenderResources {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffer: Buffer,

    render_pipeline: RenderPipeline,
    bind_group: BindGroup,

    modal: Mesh,
}

fn main() -> Result<()> {
    eframe::run_native(
        "mslicer",
        NativeOptions::default(),
        Box::new(|cc| {
            let render_state = cc.wgpu_render_state.as_ref().unwrap();
            let device = &render_state.device;

            let mut test_modal_file = File::open("teapot.stl").unwrap();
            let test_modal = slicer::mesh::load_mesh(&mut test_modal_file, "stl").unwrap();

            let vertex_buffer = device.create_buffer(&BufferDescriptor {
                label: None,
                size: test_modal.vertices.len() as u64 * std::mem::size_of::<ModelVertex>() as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let index_buffer = device.create_buffer(&BufferDescriptor {
                label: None,
                size: 3 * test_modal.faces.len() as u64 * std::mem::size_of::<u32>() as u64,
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

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
                depth_stencil: None,
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
                    vertex_buffer,
                    index_buffer,
                    uniform_buffer,

                    render_pipeline,
                    bind_group,

                    modal: test_modal,
                });

            Box::new(App {
                camera: Camera {
                    eye: Point3::new(0.0, -50.0, 5.0),
                    target: Point3::new(0.0, 50.0, 0.0),
                    up: Vector3::new(0.0, 1.0, 0.0),
                    fovy: 25.0,
                    znear: 0.1,
                    zfar: 100.0,
                },
            })
        }),
    )
    .unwrap();

    Ok(())
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("mslicer");
                ui.separator();
                if ui.button("Organize windows").clicked() {
                    ui.ctx().memory_mut(|mem| mem.reset_areas());
                }
            });
        });

        Window::new("Controls").show(ctx, |ui| {});

        CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                let (rect, _response) = ui.allocate_exact_size(ui.available_size(), Sense::drag());

                let callback = Callback::new_paint_callback(
                    rect,
                    WorkspaceRenderCallback {
                        transform: self
                            .camera
                            .build_view_projection_matrix(rect.width() / rect.height()),
                    },
                );
                ui.painter().add(callback);
            });
    }
}

impl Camera {
    fn build_view_projection_matrix(&self, aspect: f32) -> Matrix4<f32> {
        const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.0, 0.0, 0.0, 1.0,
        );

        let fov = self.fovy * std::f32::consts::PI / 180.0;

        let view = Matrix4::look_at_rh(&self.eye, &self.target, &self.up);
        let proj = Matrix4::new_perspective(aspect, fov, self.znear, self.zfar);
        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

struct WorkspaceRenderCallback {
    transform: Matrix4<f32>,
}

impl CallbackTrait for WorkspaceRenderCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
        egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources = callback_resources
            .get::<WorkspaceRenderResources>()
            .unwrap();

        let vertices = resources
            .modal
            .vertices
            .iter()
            .map(|v| ModelVertex {
                position: [v.x, v.y, v.z, 1.0],
                tex_coords: [0.0, 0.0],
            })
            .collect::<Vec<ModelVertex>>();

        queue.write_buffer(&resources.vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        queue.write_buffer(
            &resources.index_buffer,
            0,
            bytemuck::cast_slice(&resources.modal.faces),
        );

        queue.write_buffer(&resources.uniform_buffer, 0, &self.to_wgsl());

        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut egui_wgpu::wgpu::RenderPass<'a>,
        callback_resources: &'a egui_wgpu::CallbackResources,
    ) {
        let resources = callback_resources
            .get::<WorkspaceRenderResources>()
            .unwrap();

        render_pass.set_pipeline(&resources.render_pipeline);
        render_pass.set_bind_group(0, &resources.bind_group, &[]);
        render_pass.set_vertex_buffer(0, resources.vertex_buffer.slice(..));
        render_pass.set_index_buffer(resources.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..(3 * resources.modal.faces.len() as u32), 0, 0..1);
    }
}

impl WorkspaceRenderCallback {
    fn to_wgsl(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(bytemuck::cast_slice(&self.transform.as_slice()));
        out
    }
}
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 4],
    pub tex_coords: [f32; 2],
    // pub normal: [f32; 3],
}
