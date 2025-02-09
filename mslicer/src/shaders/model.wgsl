@group(0) @binding(0) var<uniform> context: Context;

struct Context {
    transform: mat4x4<f32>,
    model_transform: mat4x4<f32>,
    model_color: vec4<f32>,
    camera_position: vec3<f32>,
    camera_target: vec3<f32>,
    render_style: u32,
}

struct VertexInput {
    @location(0)
    position: vec4<f32>
}

struct VertexOutput {
    @builtin(position)
    position: vec4<f32>,
    @location(1)
    world_position: vec3<f32>
};

@vertex
fn vert(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = context.transform * in.position;
    out.world_position = (context.model_transform * in.position).xyz;
    return out;
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    let dy = dpdy(in.world_position);
    let dx = dpdx(in.world_position);
    let normal = normalize(cross(dy, dx));

    if context.render_style == 0 {
        return vec4<f32>(normal, 1.0);
    } else {
        let camera_direction = normalize(context.camera_position + context.camera_target);

        let diffuse = max(dot(normal, camera_direction), 0.0);

        let reflect_dir = reflect(-camera_direction, normal);
        let specular = pow(max(dot(camera_direction, reflect_dir), 0.0), 32.0);

        let intensity = (diffuse + specular + 0.1) * context.model_color.rgb;
        return vec4<f32>(intensity, context.model_color.a);
    }
}
