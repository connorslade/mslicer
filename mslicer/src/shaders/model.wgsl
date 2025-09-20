@group(0) @binding(0) var<uniform> context: Context;

const STYLE_NORMAL: u32 = 0;
const STYLE_RANDOM: u32 = 1;
const STYLE_RENDERD: u32 = 2;

struct Context {
    transform: mat4x4f,
    model_transform: mat4x4f,
    build_volume: vec3f,
    model_color: vec4f,
    camera_position: vec3f,
    camera_target: vec3f,
    render_style: u32,
}

struct VertexInput {
    @builtin(vertex_index) index: u32,
    @location(0) position: vec4f
}

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(1) world_position: vec3f,
    @location(2) vertex_index: u32
}

@vertex
fn vert(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = context.transform * in.position;
    out.world_position = (context.model_transform * in.position).xyz;
    out.vertex_index = in.index;
    return out;
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    let dy = dpdy(in.world_position);
    let dx = dpdx(in.world_position);
    let normal = normalize(cross(dy, dx));

    if outside_build_volume(in.world_position) {
        return vec4f(1.0, 0.0, 0.0, 1.0);
    }

    switch context.render_style {
        case STYLE_NORMAL: {
            return vec4f(normal, 1.0);
        }
        case STYLE_RANDOM: {
            seed = in.vertex_index;
            return vec4f(rand(), rand(), rand(), 1.0);
        }
        case STYLE_RENDERD: {
            let camera_direction = normalize(context.camera_position + context.camera_target);

            let diffuse = max(dot(normal, camera_direction), 0.0);

            let reflect_dir = reflect(-camera_direction, normal);
            let specular = pow(max(dot(camera_direction, reflect_dir), 0.0), 32.0);

            let intensity = (diffuse + specular + 0.1) * context.model_color.rgb;
            return vec4f(intensity, context.model_color.a);
        }
        default: {
            return vec4f(0.0);
        }
    }
}

fn outside_build_volume(pos: vec3f) -> bool {
    let build = context.build_volume / 2.0;
    return pos.x < -build.x || pos.x > build.x
        || pos.y < -build.y || pos.y > build.y
        || pos.z < 0.0 || pos.x > context.build_volume.z;
}

var<private> seed: u32 = 0u;

fn rand() -> f32 {
    seed = seed * 747796405u + 2891336453u;
    let f = f32(seed >> 9u) / f32(1u << 23u);
    return fract(f);
}
