use bytemuck::{Pod, Zeroable};
use encase::{ShaderSize, ShaderType, UniformBuffer};
use nalgebra::{Matrix4, Vector3, Vector4};
use slicer::builder::MeshBuilder;
use wgpu::{
    BindGroup, BlendState, Buffer, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites,
    DepthStencilState, Device, FragmentState, IndexFormat, MultisampleState,
    PipelineLayoutDescriptor, PrimitiveState, RenderPass, RenderPipeline, RenderPipelineDescriptor,
    TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

use crate::{
    include_shader,
    render::{
        Gcx, VERTEX_BUFFER_LAYOUT,
        consts::{
            BASE_BIND_GROUP_LAYOUT_DESCRIPTOR, BASE_UNIFORM_DESCRIPTOR, DEPTH_STENCIL_STATE,
            bind_group,
        },
        util::{ResizingBuffer, gpu_mesh_buffers},
        workspace::{
            WorkspaceRenderCallback,
            point::{overhangs::OverhangPointDispatch, target::TargetPointDispatch},
        },
    },
};

mod overhangs;
mod target;

pub struct PointDispatch {
    render_pipeline: PointPipeline,

    target_point: TargetPointDispatch,
    overhangs: OverhangPointDispatch,
}

const INSTANCE_BUFFER_LAYOUT: VertexBufferLayout = VertexBufferLayout {
    array_stride: Point::SHADER_SIZE.get(),
    step_mode: VertexStepMode::Instance,
    attributes: &[
        VertexAttribute {
            format: VertexFormat::Float32x3,
            offset: 0,
            shader_location: 1,
        },
        VertexAttribute {
            format: VertexFormat::Float32,
            offset: 12,
            shader_location: 2,
        },
        VertexAttribute {
            format: VertexFormat::Float32x4,
            offset: 16,
            shader_location: 3,
        },
    ],
};

pub struct PointPipeline {
    render_pipeline: RenderPipeline,
    bind_group: BindGroup,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffer: Buffer,
    instance_buffer: ResizingBuffer,

    index_count: u32,
    instance_count: u32,
}

#[derive(ShaderType)]
pub struct Point {
    pub position: Vector3<f32>,
    pub radius: f32,
    pub color: Vector4<f32>,
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct PointInstance {
    position: [f32; 3],
    radius: f32,
    color: [f32; 4],
}

#[derive(ShaderType)]
struct PointUniforms {
    transform: Matrix4<f32>,
}

trait PointGenerator {
    fn generate_points(&mut self, resources: &WorkspaceRenderCallback);
    fn points(&self) -> &[Point];
}

impl PointDispatch {
    pub fn new(device: &Device, texture: TextureFormat) -> Self {
        Self {
            render_pipeline: PointPipeline::new(device, texture),

            target_point: TargetPointDispatch::new(),
            overhangs: OverhangPointDispatch::new(),
        }
    }

    pub fn prepare(&mut self, gcx: &Gcx, resources: &WorkspaceRenderCallback) {
        let dispatches: &mut [&mut dyn PointGenerator] =
            &mut [&mut self.target_point, &mut self.overhangs];
        for dispatch in dispatches.iter_mut() {
            dispatch.generate_points(resources);
        }

        let points = &[self.target_point.points(), self.overhangs.points()][..];
        self.render_pipeline.prepare(gcx, resources, points);
    }

    pub fn paint(&self, render_pass: &mut RenderPass) {
        self.render_pipeline.paint(render_pass);
    }
}

impl PointPipeline {
    pub fn new(device: &Device, texture: TextureFormat) -> Self {
        let shader = device.create_shader_module(include_shader!("point.wgsl"));

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            size: PointUniforms::SHADER_SIZE.get(),
            ..BASE_UNIFORM_DESCRIPTOR
        });

        let (bind_group_layout, bind_group) = bind_group(
            device,
            BASE_BIND_GROUP_LAYOUT_DESCRIPTOR,
            [uniform_buffer.as_entire_binding()],
        );

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
                buffers: &[VERTEX_BUFFER_LAYOUT, INSTANCE_BUFFER_LAYOUT],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: None,
                targets: &[Some(ColorTargetState {
                    format: texture,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::all(),
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: Some(DepthStencilState {
                depth_write_enabled: false,
                ..DEPTH_STENCIL_STATE
            }),
            multisample: MultisampleState {
                count: 4,
                ..Default::default()
            },
            multiview: None,
            cache: None,
        });

        let mut builder = MeshBuilder::new();
        builder.add_sphere(Vector3::zeros(), 1.0, 20);
        let mesh = builder.build();

        let index_count = mesh.face_count() as u32 * 3;
        let (vertex_buffer, index_buffer) = gpu_mesh_buffers(device, &mesh);

        Self {
            render_pipeline,
            bind_group,

            vertex_buffer,
            index_buffer,
            uniform_buffer,
            instance_buffer: ResizingBuffer::new(device, BufferUsages::VERTEX),

            index_count,
            instance_count: 0,
        }
    }
}

impl PointPipeline {
    pub fn prepare(&mut self, gcx: &Gcx, resources: &WorkspaceRenderCallback, points: &[&[Point]]) {
        let points = (points.iter())
            .flat_map(|x| x.iter())
            .map(|x| x.to_instance())
            .collect::<Vec<_>>();
        self.instance_count = points.len() as u32;
        self.instance_buffer.write_slice(gcx, &points);

        let mut buffer = UniformBuffer::new(Vec::new());
        let transform = resources.transform;
        buffer.write(&PointUniforms { transform }).unwrap();
        gcx.queue
            .write_buffer(&self.uniform_buffer, 0, &buffer.into_inner());
    }

    pub fn paint(&self, render_pass: &mut RenderPass) {
        if self.instance_count == 0 {
            return;
        }

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..self.instance_count);
    }
}

impl Point {
    fn to_instance(&self) -> PointInstance {
        PointInstance {
            position: self.position.into(),
            radius: self.radius,
            color: self.color.into(),
        }
    }
}
