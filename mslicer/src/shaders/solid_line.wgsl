@group(0) @binding(0) var<uniform> context: Context;

struct Context {
    transform: mat4x4<f32>,
}

struct VertexOutput {
    @builtin(position)
    position: vec4<f32>,
    @location(0)
    color: vec3<f32>,
};

@vertex
fn vert(
    @location(0) position: vec4<f32>,
    @location(1) color: vec3<f32>
) -> VertexOutput {
    var out: VertexOutput;
    out.position = context.transform * position;
    out.color = color;
    return out;
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}