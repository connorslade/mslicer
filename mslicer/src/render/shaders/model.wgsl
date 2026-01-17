@group(0) @binding(0) var<uniform> context: Context;

const STYLE_NORMAL: u32 = 0;
const STYLE_RANDOM: u32 = 1;
const STYLE_RENDERED: u32 = 2;

const OOB_COLOR: vec3f = vec3f(1.0, 0.0, 0.0);
const OVERHANG_COLOR: vec3f = vec3f(0.67, 0.65, 0.38);

struct Context {
    transform: mat4x4f,
    model_transform: mat4x4f,
    build_volume: vec3f,
    model_color: vec3f,
    camera_position: vec3f,
    camera_target: vec3f,
    render_style: u32,
    overhang_angle: f32
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
fn frag(
   @builtin(front_facing) is_front: bool,
   in: VertexOutput
) -> @location(0) vec4f {
    let normal = screen_normal(in.world_position);

    switch context.render_style {
        case STYLE_NORMAL: {
            return vec4f(normal, 1.0);
        }
        case STYLE_RANDOM: {
            seed = in.vertex_index;
            return vec4f(rand(), rand(), rand(), 1.0);
        }
        case STYLE_RENDERED: {
            var color = context.model_color;
            if bitcast<u32>(context.overhang_angle) != 0xFFFFFFFF {
                color = mix(color, OVERHANG_COLOR, 1.0 - smoothstep(0, context.overhang_angle, acos(-normal.z)));
            }

            color = select(vec3f(.5), color, is_front);
            color = select(color, OOB_COLOR, outside_build_volume(in.world_position));

            let camera_direction = normalize(context.camera_position + context.camera_target);
            let intensity = blinn_phong(normal, camera_direction);
            return vec4f(intensity * color, 1.0);
        }
        default: {
            return vec4f();
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
