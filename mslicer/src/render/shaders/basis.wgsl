@group(0) @binding(0) var<uniform> ctx: Context;

const COLORS: array<vec3f, 3> = array(
    vec3(0.8118, 0.0000, 0.0039),
    vec3(0.0000, 0.8118, 0.1529),
    vec3(0.0627, 0.3176, 0.9843),
);

struct Context {
    transform: mat4x4f
}

struct VertexInput {
    @builtin(vertex_index) index: u32,
    @location(0) position: vec4f
}

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(1) vertex_index: u32

}

@vertex
fn vert(in: VertexInput) -> VertexOutput {
    return VertexOutput(ctx.transform * in.position, in.index);
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4f {
    let axis = in.vertex_index / 176;
    return vec4(COLORS[axis], 1.0);
}
