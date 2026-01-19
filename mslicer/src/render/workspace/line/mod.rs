use encase::{ShaderSize, ShaderType, UniformBuffer};
use nalgebra::{Matrix4, Vector3};
use wgpu::{
    BindGroup, Buffer, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, Device,
    FragmentState, IndexFormat, MultisampleState, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, RenderPass, RenderPipeline, RenderPipelineDescriptor, TextureFormat,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

use crate::{
    app::App,
    include_shader,
    render::{
        Gcx,
        consts::{
            BASE_BIND_GROUP_LAYOUT_DESCRIPTOR, BASE_UNIFORM_DESCRIPTOR, DEPTH_STENCIL_STATE,
            bind_group,
        },
        util::ResizingBuffer,
        workspace::line::{
            build_plate::BuildPlateDispatch, line_support_debug::LineSupportDebugDispatch,
            normals::NormalsDispatch,
        },
    },
};

mod build_plate;
mod line_support_debug;
mod normals;

const VERTEX_BUFFER_LAYOUT: VertexBufferLayout = VertexBufferLayout {
    array_stride: LineVertex::SHADER_SIZE.get(),
    step_mode: VertexStepMode::Vertex,
    attributes: &[
        VertexAttribute {
            format: VertexFormat::Float32x4,
            offset: 0,
            shader_location: 0,
        },
        VertexAttribute {
            format: VertexFormat::Float32x3,
            offset: 4 * 4,
            shader_location: 1,
        },
    ],
};

pub struct LineDispatch {
    render_pipeline: LinePipeline,

    build_plate: BuildPlateDispatch,
    normals: NormalsDispatch,
    line_support_debug: LineSupportDebugDispatch,
}

pub struct LinePipeline {
    render_pipeline: RenderPipeline,
    bind_group: BindGroup,

    vertex_buffer: ResizingBuffer,
    index_buffer: ResizingBuffer,
    uniform_buffer: Buffer,

    vertex_count: u32,
}

#[derive(Clone)]
pub struct Line {
    start: Vector3<f32>,
    end: Vector3<f32>,

    color: Vector3<f32>,
}

#[derive(ShaderType)]
struct LineUniforms {
    transform: Matrix4<f32>,
}

#[repr(C)]
#[derive(Default, Copy, Clone, ShaderType, bytemuck::Pod, bytemuck::Zeroable)]
struct LineVertex {
    pub position: [f32; 4],
    pub color: [f32; 3],
}

trait LineGenerator {
    fn generate_lines(&mut self, app: &mut App);
    fn lines(&self) -> &[Line];
}

impl LineDispatch {
    pub fn new(device: &Device, texture: TextureFormat) -> Self {
        Self {
            render_pipeline: LinePipeline::new(device, texture),

            build_plate: BuildPlateDispatch::new(),
            normals: NormalsDispatch::new(),
            line_support_debug: LineSupportDebugDispatch::new(),
        }
    }

    pub fn prepare(&mut self, gcx: &Gcx, app: &mut App) {
        let dispatches: &mut [&mut dyn LineGenerator] = &mut [
            &mut self.build_plate,
            &mut self.normals,
            &mut self.line_support_debug,
        ];
        for dispatch in dispatches.iter_mut() {
            dispatch.generate_lines(app);
        }

        let lines = &[
            self.build_plate.lines(),
            self.normals.lines(),
            self.line_support_debug.lines(),
        ][..];
        self.render_pipeline.prepare(gcx, app, lines);
    }

    pub fn paint(&self, render_pass: &mut RenderPass) {
        self.render_pipeline.paint(render_pass);
    }
}

impl LinePipeline {
    pub fn new(device: &Device, texture: TextureFormat) -> Self {
        let shader = device.create_shader_module(include_shader!("line.wgsl"));

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            size: LineUniforms::SHADER_SIZE.get(),
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
            primitive: PrimitiveState {
                polygon_mode: PolygonMode::Line,
                ..Default::default()
            },
            depth_stencil: Some(DEPTH_STENCIL_STATE),
            multisample: MultisampleState {
                count: 4,
                ..Default::default()
            },
            multiview: None,
            cache: None,
        });

        Self {
            render_pipeline,
            bind_group,

            vertex_buffer: ResizingBuffer::new(device, BufferUsages::VERTEX),
            index_buffer: ResizingBuffer::new(device, BufferUsages::INDEX),
            uniform_buffer,

            vertex_count: 0,
        }
    }
}

impl LinePipeline {
    pub fn prepare(&mut self, gcx: &Gcx, app: &mut App, lines: &[&[Line]]) {
        let vertex = (lines.iter())
            .flat_map(|x| x.iter())
            .flat_map(Line::to_vertex)
            .collect::<Vec<_>>();
        let index = (0..vertex.len() as u32).collect::<Vec<_>>();

        self.vertex_count = vertex.len() as u32;
        self.vertex_buffer.write_slice(gcx, &vertex);
        self.index_buffer.write_slice(gcx, &index);

        let mut buffer = UniformBuffer::new(Vec::new());
        let transform = app.view_projection();
        buffer.write(&LineUniforms { transform }).unwrap();
        gcx.queue
            .write_buffer(&self.uniform_buffer, 0, &buffer.into_inner());
    }

    pub fn paint(&self, render_pass: &mut RenderPass) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.vertex_count, 0, 0..1);
    }
}

impl Line {
    pub fn new(start: Vector3<f32>, end: Vector3<f32>) -> Self {
        Self {
            start,
            end,
            color: Vector3::zeros(),
        }
    }

    pub fn color(mut self, color: Vector3<f32>) -> Self {
        self.color = color;
        self
    }

    fn to_vertex(&self) -> [LineVertex; 3] {
        let color = [self.color.x, self.color.y, self.color.z];
        [
            LineVertex {
                position: [self.start.x, self.start.y, self.start.z, 1.0],
                color,
            },
            LineVertex {
                position: [self.end.x, self.end.y, self.end.z, 1.0],
                color,
            },
            LineVertex {
                position: [self.start.x, self.start.y, self.start.z, 1.0],
                color,
            },
        ]
    }
}
