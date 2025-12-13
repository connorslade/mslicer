@group(0) @binding(0) var<uniform> context: Context;

struct Context {
    transform: mat4x4f,
}

struct VertexInput {
    @location(0) position: vec4f,
    @location(1) color: vec3f
}

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) color: vec3f
}

@vertex
fn vert(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = context.transform * in.position;
    out.color = in.color;
    return out;
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4f {
    return vec4f(in.color, 1.0);
}
