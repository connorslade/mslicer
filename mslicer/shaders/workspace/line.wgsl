@group(0) @binding(0) var<uniform> transform: mat4x4f;

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
    return VertexOutput(transform * in.position, in.color);
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4f {
    return vec4f(in.color, 1.0);
}
