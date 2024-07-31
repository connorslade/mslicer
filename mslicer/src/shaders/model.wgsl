@group(0) @binding(0) var<uniform> context: Context;

struct Context {
    transform: mat4x4<f32>,
    model_transform: mat4x4<f32>,
    model_color: vec4<f32>,
    camera_position: vec3<f32>,
    camera_target: vec3<f32>,
    render_style: u32,
}

struct VertexOutput {
    @builtin(position)
    position: vec4<f32>,
    @location(1)
    normal: vec3<f32>,
};

@vertex
fn vert(
    @location(0) position: vec4<f32>,
    @location(1) normal: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = context.transform * position;
    out.normal = normalize((context.model_transform * vec4(normal, 0.0)).xyz);
    return out;
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    if context.render_style == 0 {
        return vec4<f32>(in.normal, 1.0);
    } else {
        let camera_direction = normalize(context.camera_position + context.camera_target);

        let diffuse = max(dot(in.normal, camera_direction), 0.0);

        let reflect_dir = reflect(-camera_direction, in.normal);
        let specular = pow(max(dot(camera_direction, reflect_dir), 0.0), 32.0);

        let intensity = (diffuse + specular + 0.1) * context.model_color.rgb;
        return vec4<f32>(intensity, context.model_color.a);
    }
}