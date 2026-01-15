@group(0) @binding(0) var<uniform> context: Context;

struct Context {
    transform: mat4x4f,
    camera_direction: vec3f
}

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(1) world_position: vec3f,
}

@vertex
fn vert(@location(0) position: vec4f) -> VertexOutput {
    return VertexOutput(context.transform * position, vec3f(0));
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4f {
    let intensity = blinn_phong(screen_normal(in.world_position), context.camera_direction);
    return vec4(vec3(intensity), 0.5);
}
