use std::ops::Deref;

use common::container::Run;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, Buffer, BufferBindingType, BufferUsages, CommandEncoder, ComputePassDescriptor,
    ComputePipeline, ComputePipelineDescriptor, Device, PipelineLayoutDescriptor,
    PushConstantRange, ShaderStages,
};

use crate::{
    include_shader,
    render::{Gcx, util::ResizingBuffer},
};

pub struct DecompressPass {
    pipeline: ComputePipeline,
}

pub struct DecompressedBuffer {
    compressed: ResizingBuffer,
    uncompressed: ResizingBuffer,
}

impl DecompressPass {
    pub fn new(device: &Device) -> Self {
        let shader = &device.create_shader_module(include_shader!("slice_preview/decompress.wgsl"));

        let bind_group_layout = &device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[PushConstantRange {
                stages: ShaderStages::COMPUTE,
                range: 0..4,
            }],
        });

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: shader,
            entry_point: None,
            compilation_options: Default::default(),
            cache: None,
        });

        Self { pipeline }
    }

    pub fn decompress(
        &mut self,
        gcx: &Gcx,
        encoder: &mut CommandEncoder,
        buffer: &mut DecompressedBuffer,
        runs: &[Run],
    ) {
        let mut data = Vec::new();
        let mut i = 0_u32;

        for run in runs {
            if run.value > 0 {
                let mut remaining = run.length;
                while remaining > 0 {
                    let take = remaining.min(0x2000);
                    remaining -= take;

                    data.push(i);
                    data.push((take as u32) << 8 | run.value as u32);
                    i += take as u32;
                }
            } else {
                i += run.length as u32;
            }
        }

        buffer.compressed.write_slice(gcx, &data);
        buffer.uncompressed.resize(gcx, i as u64);

        let bind_group = gcx.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: buffer.compressed.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: buffer.uncompressed.as_entire_binding(),
                },
            ],
        });

        encoder.clear_buffer(&buffer.uncompressed, 0, None);
        let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });

        let run_count = data.len() / 2;
        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.set_push_constants(0, bytemuck::cast_slice(&[run_count as u32]));
        compute_pass.dispatch_workgroups(run_count.div_ceil(64) as u32, 1, 1);
    }
}

impl DecompressedBuffer {
    pub fn new(device: &Device) -> Self {
        let compressed =
            ResizingBuffer::new_sized(device, BufferUsages::STORAGE | BufferUsages::COPY_DST, 4);
        let uncompressed = ResizingBuffer::new_sized(device, BufferUsages::STORAGE, 4);

        Self {
            compressed,
            uncompressed,
        }
    }
}

impl Deref for DecompressedBuffer {
    type Target = Buffer;

    fn deref(&self) -> &Self::Target {
        &self.uncompressed
    }
}
