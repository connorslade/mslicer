@group(0) @binding(0) var<uniform> context: Context;
@group(0) @binding(1) var<storage, read> selected: array<u32>;

const STYLE_NORMAL: u32 = 0;
const STYLE_RANDOM: u32 = 1;
const STYLE_RENDERED: u32 = 2;

const SELECTED_COLOR: vec3f = vec3f(0.2, 0.2, 1.0);
const OOB_COLOR: vec3f = vec3f(1.0, 0.0, 0.0);
const OVERHANG_COLOR: vec3f = vec3f(0.67, 0.65, 0.38);

struct Context {
    transform: mat4x4f,
    model_transform: mat4x4f,
    build_volume: vec3f,
    model_color: vec3f,
    render_style: u32,
    overhang_angle: f32,
    id: u32,
    selected_offset: u32
}

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(1) world_position: vec3f,
}

struct FragmentOutput {
    @location(0) color: vec4f,
    @location(1) model: vec2u,
    @location(2) normal: vec4f,
    @location(3) world: vec4f,
}

@vertex
fn vert(@location(0) xyz: vec3f) -> VertexOutput {
    let position = vec4(xyz, 1.0);
    return VertexOutput(
        context.transform * position,
        (context.model_transform * position).xyz,
    );
}

@fragment
fn frag(
   @builtin(front_facing) is_front: bool,
   @builtin(primitive_index) index: u32,
   in: VertexOutput
) -> FragmentOutput {
    let normal = screen_normal(in.world_position);
    return FragmentOutput(
        render(is_front, in, index, normal),
        vec2(context.id, index),
        vec4(normal, 0.0),
        vec4(in.world_position, 0.0)
    );
}

fn render(is_front: bool, in: VertexOutput, index: u32, normal: vec3f) -> vec4f {
    switch context.render_style {
        case STYLE_NORMAL: {
            return vec4f(normal * 0.5 + 0.5, 1.0);
        }
        case STYLE_RANDOM: {
            seed = index;
            return vec4f(vec3f(rand(), rand(), rand()), 1.0);
        }
        case STYLE_RENDERED: {
            var color = context.model_color;
            if bitcast<u32>(context.overhang_angle) != 0xFFFFFFFF {
                color = mix(color, OVERHANG_COLOR, 1.0 - smoothstep(0, context.overhang_angle, acos(-normal.z)));
            }

            color = select(color, SELECTED_COLOR, is_selected(index));
            color = select(vec3f(.5), color, is_front);
            color = select(color, OOB_COLOR, outside_build_volume(in.world_position));

            return vec4f(color, 1.0);
        }
        default: {
            return vec4f();
        }
    }
}

fn is_selected(face: u32) -> bool {
    let word = context.selected_offset + face / 32;
    let bit = face % 32;
    return ((selected[word] >> bit) & 0x01) != 0;
}

fn outside_build_volume(pos: vec3f) -> bool {
    let build = context.build_volume / 2.0;
    return pos.x < -build.x || pos.x > build.x
        || pos.y < -build.y || pos.y > build.y
        || pos.z < -0.01    || pos.z > context.build_volume.z;
}
