use std::{fs::File, mem};

use anyhow::Result;
use eframe::NativeOptions;
use egui::{CentralPanel, Frame, Sense, TopBottomPanel, Vec2, Window};
use egui_wgpu::{Callback, CallbackTrait};
use slicer::mesh::Mesh;
use wgpu::{
    BindGroup, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferAddress, BufferBindingType, BufferDescriptor, BufferSize, BufferUsages, ColorTargetState, ColorWrites, Face, FragmentState, IndexFormat, MultisampleState, PipelineLayoutDescriptor, PrimitiveState, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderStages, TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode
};

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Bgra8Unorm;

struct App {}

struct WorkspaceRenderResources {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
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

            let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[],
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
                entries: &[],
            });

            render_state
                .renderer
                .write()
                .callback_resources
                .insert(WorkspaceRenderResources {
                    vertex_buffer,
                    index_buffer,
                    render_pipeline,
                    bind_group,
                    modal: test_modal,
                });

            Box::new(App {})
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

        CentralPanel::default().show(ctx, |ui| {
            Frame::canvas(ui.style()).show(ui, |ui| {
                let (rect, response) = ui.allocate_exact_size(Vec2::splat(300.0), Sense::drag());

                let callback = Callback::new_paint_callback(rect, WorkspaceRenderCallback);
                ui.painter().add(callback);
            });
        });
    }
}

struct WorkspaceRenderCallback;

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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 4],
    pub tex_coords: [f32; 2],
    // pub normal: [f32; 3],
}
