@group(0) @binding(0) var<uniform> context: Context;
@group(0) @binding(1) var<storage, read> layer: array<u32>;

struct Context {
    dimensions: vec2<u32>,
    offset: vec2<f32>,
    aspect: f32, // width / height
    scale: f32,
}

struct VertexOutput {
    @builtin(position)
    camera_position: vec4<f32>,
    @location(0)
    position: vec4<f32>,
};

@vertex
fn vert(@location(0) position: vec4<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.camera_position = position;
    out.position = position;
    return out;
}

fn index(x: u32, y: u32) -> f32 {
    let byte_idx = (y * context.dimensions.x + x);
    let array_idx = byte_idx / 4;
    let shift = (byte_idx % 4) * 8;

    let value = (layer[array_idx] >> shift) & 0xFF;
    return f32(value) / 255.0;
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
let aspect = context.aspect * f32(context.dimensions.y) / f32(context.dimensions.x);
    let pos = vec2(
        in.position.x * context.scale * aspect,
        in.position.y * context.scale
    ) / 2.0 + 0.5;

    let value = index(
        u32(pos.x * f32(context.dimensions.x) + context.offset.x),
        u32(pos.y * f32(context.dimensions.y) + context.offset.y)
    );
    return vec4<f32>(value, value, value, 1.0);
}
