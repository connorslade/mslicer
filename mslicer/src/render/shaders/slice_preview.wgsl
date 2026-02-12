@group(0) @binding(0) var<uniform> context: Context;
@group(0) @binding(1) var<storage, read> layer: array<u32>;
@group(0) @binding(2) var<storage, read> annotations: array<u32>;

const GRID_WIDTH: f32 = 2.0;

const POINTS = array(
    vec2(-1.0, -1.0),
    vec2( 3.0, -1.0),
    vec2(-1.0,  3.0)
);
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
    let uv = vec2(in.position.x * aspect, in.position.y) * context.scale / 2.0 + 0.5;
    let pos = uv * vec2f(context.dimensions) + context.offset;

    // let pixel = fwidth(pos) / 2;
    // let dist = min(fract(pos), 1.0 - fract(pos));

    // let outer_edge = pixel * (GRID_WIDTH + 1);
    // let inner_edge = pixel * (GRID_WIDTH - 1);
    // let grid = max(
    //     smoothstep(outer_edge.x, inner_edge.x, dist.x),
    //     smoothstep(outer_edge.y, inner_edge.y, dist.y)
    // ) * saturate(invMix(0.0221, 0.0156, context.scale));

    // let value = pixel(vec2u(pos));
    // let out = mix(value, 0.5, grid);
    // return vec4f(vec3f(out), 1.0);

    return vec4f(pixel(vec2u(pos)), 1.0);
}

fn pixel(pos: vec2u) -> vec3f {
    if pos.x >= context.dimensions.x || pos.y >= context.dimensions.y {
        return vec3f(0);
    }

    let brightness = index_slice(pos);
    let color = index_annotation(pos);

    return vec3f(brightness) * color;
}

fn index_slice(pos: vec2u) -> f32 {
    let byte_idx = (pos.y * context.dimensions.x + pos.x);
    let array_idx = byte_idx / 4;
    let shift = (byte_idx % 4) * 8;

    let value = (layer[array_idx] >> shift) & 0xFF;
    return f32(value) / 255.0;
}

fn index_annotation(pos: vec2u) -> vec3f {
    // let nibble_idx = (pos.y * context.dimensions.x + pos.x);
    // let array_idx = nibble_idx / 8;
    // let shift = (nibble_idx % 8) * 4;

    // let value = (annotations[array_idx] >> shift) & 0xF;

    let byte_idx = (pos.y * context.dimensions.x + pos.x);
    let array_idx = byte_idx / 4;
    let shift = (byte_idx % 4) * 8;

    let value = (annotations[array_idx] >> shift) & 0xFF;
    return ANNOTATION_COLORS[value];
}

fn invMix(a: f32, b: f32, value: f32) -> f32 {
    return (value - a) / (b - a);
}
