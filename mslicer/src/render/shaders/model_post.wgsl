@group(0) @binding(0) var<uniform> context: Context;
@group(0) @binding(1) var texture: texture_2d<f32>;
@group(0) @binding(2) var normal: texture_multisampled_2d<f32>;
@group(0) @binding(3) var depth: texture_depth_multisampled_2d;
@group(0) @binding(4) var texture_sampler: sampler;

struct Context {
    view: mat4x4f,
    inv_view: mat4x4f,
}

struct VertexOutput {
    @builtin(position) camera_position: vec4f,
    @location(0) position: vec2f,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@vertex
fn vert(@builtin(vertex_index) index: u32) -> VertexOutput {
    let position = CLIP_TRI[index];
    return VertexOutput(vec4f(vec4(position, 0, 1)), vec2f(position));
}

@fragment
fn frag(in: VertexOutput) -> FragmentOutput {
    let uv = vec2(0.0, 1.0) + (in.position * 0.5 + vec2(0.5)) * vec2f(1.0, -1.0);
    let depth = sample_depth(uv);
    let world_normal = sample_normal(uv).xyz;

    return FragmentOutput(vec4f(world_normal, color.a), depth);
}

fn sample_depth(uv: vec2f) -> f32 {
    let coord = vec2i(uv * vec2f(textureDimensions(depth)));
    return textureLoad(depth, coord, 0);
}

fn sample_normal(uv: vec2f) -> vec4f {
    let coord = vec2i(uv * vec2f(textureDimensions(normal)));
    return textureLoad(normal, coord, 0);
}

fn sample_color(uv: vec2f) -> vec4f {
    let coord = vec2i(uv * vec2f(textureDimensions(texture)));
    return textureLoad(texture, coord, 0);
}
