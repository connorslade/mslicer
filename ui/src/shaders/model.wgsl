@group(0) @binding(0) var<uniform> context: Context;

struct Context {
    transform: mat4x4<f32>,
    render_style: u32,
}

struct VertexOutput {
    @builtin(position)
    camera_position: vec4<f32>,
    @location(0)
    position: vec4<f32>,
    @location(1)
    tex_coord: vec2<f32>,
    @location(2)
    normal: vec3<f32>,
};

@vertex
fn vert(
    @location(0) position: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) normal: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.camera_position = context.transform * position;
    out.position = context.transform * position;
    out.tex_coord = tex_coord;
    out.normal = normal;
    return out;
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    if context.render_style == 0 {
        return vec4<f32>(in.normal, 1.0);
    } else {
        let light_dir = normalize(vec3<f32>(0.0, 0.0, 1.0));
        let intensity = max(dot(in.normal, light_dir), 0.0);
        return vec4<f32>(intensity, intensity, intensity, 1.0);
    }
}