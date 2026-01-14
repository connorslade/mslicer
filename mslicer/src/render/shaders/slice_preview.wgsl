@group(0) @binding(0) var<uniform> context: Context;
@group(0) @binding(1) var<storage, read> layer: array<u32>;

const GRID_WIDTH: f32 = 2.0;

struct Context {
    dimensions: vec2u,
    offset: vec2f,
    aspect: f32, // width / height
    scale: f32,
}

struct VertexOutput {
    @builtin(position)camera_position: vec4f,
    @location(0) position: vec4f,
}

@vertex
fn vert(@location(0) position: vec4f) -> VertexOutput {
    var out: VertexOutput;
    out.camera_position = position;
    out.position = position;
    return out;
}

fn index(pos: vec2u) -> f32 {
    if pos.x >= context.dimensions.x || pos.y >= context.dimensions.y {
        return 0.0;
    }

    let byte_idx = (pos.y * context.dimensions.x + pos.x);
    let array_idx = byte_idx / 4;
    let shift = (byte_idx % 4) * 8;

    let value = (layer[array_idx] >> shift) & 0xFF;
    return f32(value) / 255.0;
}

fn invMix(a: f32, b: f32, value: f32) -> f32 {
    return (value - a) / (b - a);
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4f {
    let aspect = context.aspect * f32(context.dimensions.y) / f32(context.dimensions.x);
    let uv = vec2(in.position.x * aspect, in.position.y) * context.scale / 2.0 + 0.5;
    let pos = uv * vec2f(context.dimensions) + context.offset;

    let pixel = fwidth(pos) / 2;
    let dist = min(fract(pos), 1.0 - fract(pos));

    let outer_edge = pixel * (GRID_WIDTH + 1);
    let inner_edge = pixel * (GRID_WIDTH - 1);
    let grid = max(
        smoothstep(outer_edge.x, inner_edge.x, dist.x),
        smoothstep(outer_edge.y, inner_edge.y, dist.y)
    ) * saturate(invMix(0.0221, 0.0156, context.scale));

    let value = index(vec2u(pos));
    let out = mix(value, 0.5, grid);
    return vec4f(vec3f(out), 1.0);
}
