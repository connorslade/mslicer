@group(0) @binding(0) var<uniform> context: Context;
@group(0) @binding(1) var<storage, read> layer: array<u32>;
@group(0) @binding(2) var<storage, read> annotations: array<u32>;

const GRID_WIDTH: f32 = 2.0;

const POINTS = array(
    vec2(-1.0, -1.0),
    vec2( 3.0, -1.0),
    vec2(-1.0,  3.0)
);

const BACKGROUND_COLOR = vec3f(0.106);
const ANNOTATION_COLORS = array(
    vec3f(1.000, 1.000, 1.000), // (00) No annotation
    vec3f(0.624, 0.176, 0.212), // (01) Island
    vec3f(1.000, 1.000, 1.000), // (10) Unused
    vec3f(1.000, 1.000, 1.000), // (11) Unused
);

struct Context {
    dimensions: vec2u,
    offset: vec2f,
    aspect: f32, // width / height
    scale: f32,
}

struct VertexOutput {
    @builtin(position) camera_position: vec4f,
    @location(0) position: vec2f,
}

@vertex
fn vert(@builtin(vertex_index) index: u32) -> VertexOutput {
    let position = POINTS[index];
    return VertexOutput(vec4f(vec4(position, 0, 1)), vec2f(position));
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4f {
    let aspect = context.aspect * f32(context.dimensions.y) / f32(context.dimensions.x);
    let uv = vec2(in.position.x * aspect, in.position.y) / context.scale / 2.0 + 0.5;
    let pos = vec2i(uv * vec2f(context.dimensions) + context.offset);

    let upos = vec2u(pos);
    if pos.x < 0 || pos.y < 0
        || upos.x >= context.dimensions.x
        || upos.y >= context.dimensions.y {
        return vec4f(BACKGROUND_COLOR, 1.0);
    }

    let brightness = index_slice(upos);
    let color = index_annotation(upos);
    return vec4f(vec3f(brightness) * color, 1.0);
}


struct Index {
    array_idx: u32,
    shift: u32
}

fn index(pos: vec2u) -> Index {
    let byte_idx = (pos.y * context.dimensions.x + pos.x);
    return Index(byte_idx / 4, (byte_idx % 4) * 8);
}

fn index_slice(pos: vec2u) -> f32 {
    let index = index(pos);
    let value = (layer[index.array_idx] >> index.shift) & 0xFF;
    return f32(value) / 255.0;
}

fn index_annotation(pos: vec2u) -> vec3f {
    let index = index(pos);
    let value = (annotations[index.array_idx] >> index.shift) & 0xFF;
    return ANNOTATION_COLORS[value];
}
