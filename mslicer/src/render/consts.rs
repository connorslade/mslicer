use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
    BufferBindingType, BufferDescriptor, BufferUsages, CompareFunction, DepthBiasState,
    DepthStencilState, Device, FilterMode, SamplerDescriptor, ShaderStages, StencilFaceState,
    StencilState,
};

use crate::DEPTH_TEXTURE_FORMAT;

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

pub const STORAGE_BIND_GROUP_LAYOUT_ENTRY: BindGroupLayoutEntry = BindGroupLayoutEntry {
    binding: 1,
    visibility: ShaderStages::FRAGMENT,
    ty: BindingType::Buffer {
        ty: BufferBindingType::Storage { read_only: true },
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

pub const DEPTH_STENCIL_STATE: DepthStencilState = DepthStencilState {
    format: DEPTH_TEXTURE_FORMAT,
    depth_write_enabled: true,
    depth_compare: CompareFunction::Less,
    stencil: StencilState {
        front: StencilFaceState::IGNORE,
        back: StencilFaceState::IGNORE,
        read_mask: 0,
        write_mask: 0,
    },
    bias: DepthBiasState {
        constant: 0,
        slope_scale: 0.0,
        clamp: 0.0,
    },
};

pub const FILTERING_SAMPLER: SamplerDescriptor = SamplerDescriptor {
    label: None,
    address_mode_u: AddressMode::ClampToEdge,
    address_mode_v: AddressMode::ClampToEdge,
    address_mode_w: AddressMode::ClampToEdge,
    mag_filter: FilterMode::Linear,
    min_filter: FilterMode::Linear,
    mipmap_filter: FilterMode::Linear,
    lod_min_clamp: 0.0,
    lod_max_clamp: 0.0,
    compare: None,
    anisotropy_clamp: 1,
    border_color: None,
};

pub fn bind_group<'a, const N: usize>(
    device: &Device,
    layout_descriptor: BindGroupLayoutDescriptor,
    resources: [BindingResource<'a>; N],
) -> (BindGroupLayout, BindGroup) {
    let bind_group_layout = device.create_bind_group_layout(&layout_descriptor);
    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &resources.map(|x| BindGroupEntry {
            binding: 0,
            resource: x,
        }),
    });

    (bind_group_layout, bind_group)
}
