use wgpu::{
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType,
    BufferDescriptor, BufferUsages, ShaderStages,
};

pub const BASE_UNIFORM_DESCRIPTOR: BufferDescriptor = BufferDescriptor {
    label: None,
    size: 0,
    usage: BufferUsages::UNIFORM.union(BufferUsages::COPY_DST),
    mapped_at_creation: false,
};

pub const UNIFORM_BIND_GROUP_LAYOUT_ENTRY: BindGroupLayoutEntry = BindGroupLayoutEntry {
    binding: 0,
    visibility: ShaderStages::VERTEX.union(ShaderStages::FRAGMENT),
    ty: BindingType::Buffer {
        ty: BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
    },
    count: None,
};

pub const BASE_BIND_GROUP_LAYOUT_DESCRIPTOR: BindGroupLayoutDescriptor =
    BindGroupLayoutDescriptor {
        label: None,
        entries: &[UNIFORM_BIND_GROUP_LAYOUT_ENTRY],
    };
