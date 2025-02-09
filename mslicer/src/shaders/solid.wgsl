@group(0) @binding(0) var<uniform> context: Context;

struct Context {
    transform: mat4x4<f32>,
    color: vec4<f32>
}

struct VertexOutput {
    @builtin(position)
    position: vec4<f32>,
};

@vertex
fn vert(@location(0) position: vec4<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.position = context.transform * position;
    return out;
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    return context.color;
}
