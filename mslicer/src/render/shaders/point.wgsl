@group(0) @binding(0) var<uniform> context: Context;

struct Context {
    transform: mat4x4f,
}

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(1) color: vec4f
}

struct Instance {
    @location(1) position: vec3f,
    @location(2) radius: f32,
    @location(3) color: vec4f,
}

@vertex
fn vert(@location(0) vert: vec4f, inst: Instance) -> VertexOutput {
    let world = vert.xyz * inst.radius + inst.position;
    return  VertexOutput(context.transform * vec4(world, vert.w), inst.color);
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
