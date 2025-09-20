@group(0) @binding(0) var<uniform> context: Context;
@group(0) @binding(1) var<storage, read> layer: array<u32>;

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
    let byte_idx = (pos.y * context.dimensions.x + pos.x);
    let array_idx = byte_idx / 4;
    let shift = (byte_idx % 4) * 8;

    let value = (layer[array_idx] >> shift) & 0xFF;
    return f32(value) / 255.0;
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4f {
    let aspect = context.aspect * f32(context.dimensions.y) / f32(context.dimensions.x);
    let pos = vec2(in.position.x * aspect, in.position.y) * context.scale / 2.0 + 0.5;
    let pixel = pos * vec2f(context.dimensions) + context.offset;

    let value = index(vec2u(pixel));
    return vec4f(vec3f(value), 1.0);
}
