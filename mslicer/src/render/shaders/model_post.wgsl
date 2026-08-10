@group(0) @binding(0) var<uniform> context: Context;
@group(0) @binding(1) var texture: texture_2d<f32>;
@group(0) @binding(2) var depth: texture_depth_multisampled_2d;
@group(0) @binding(3) var texture_sampler: sampler;

// todo: move to common.wgsl
const POINTS = array(
    vec2( 1.0,  1.0),
    vec2(-3.0,  1.0),
    vec2( 1.0, -3.0)
);

struct Context {
    x: f32
}

struct VertexOutput {
    @builtin(position) camera_position: vec4f,
    @location(0) position: vec2f,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
};

@vertex
fn vert(@builtin(vertex_index) index: u32) -> VertexOutput {
    let position = POINTS[index];
    return VertexOutput(vec4f(vec4(position, 0, 1)), vec2f(position));
}

@fragment
fn frag(in: VertexOutput) -> FragmentOutput {
    let uv = vec2(0.0, 1.0) + (in.position * 0.5 + vec2(0.5)) * vec2f(1.0, -1.0);
    let depth = sample_depth(uv);
    let color = textureSample(texture, texture_sampler, uv);

    return FragmentOutput(color, depth);
}

fn sample_depth(uv: vec2f) -> f32 {
    let dims = textureDimensions(depth);
    let coord = vec2<i32>(uv * vec2f(dims));
    return textureLoad(depth, coord, 0);
}
