@group(0) @binding(0) var<uniform> context: Context;
@group(0) @binding(1) var<storage, read> layer: array<u32>;
@group(0) @binding(2) var<storage, read> annotations: array<u32>;

const ANN_ERROR: u32 = 1u << 7u;
const ANN_WARN : u32 = 1u << 6u;
const ANN_INFO : u32 = 1u << 5u;
const ANN_DEBUG: u32 = 1u << 4u;

const ANN_ISLAND: u32 = 1u << 0u;

const COLOR_ERROR = vec4(1.0, 0.5, 0.5, 1.0);
const COLOR_WARN  = vec4(0.9921875, 0.9921875, 0.87109375, 1.0);
const COLOR_INFO  = vec4(0.67578125, 0.84375, 0.8984375, 1.0);
const COLOR_DEBUG = vec4(0.5, 0.5, 0.5, 1.0);


struct Context {
    dimensions: vec2<u32>,
    offset: vec2<f32>,
    aspect: f32, // width / height
    scale: f32,
    show_hide: u32,
}

struct VertexOutput {
    @builtin(position)
    camera_position: vec4<f32>,
    @location(0)
    position: vec4<f32>,
};

@vertex
fn vert(@location(0) position: vec4<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.camera_position = position;
    out.position = position;
    return out;
}

fn index(x: u32, y: u32) -> f32 {
    let byte_idx = (y * context.dimensions.x + x);
    let array_idx = byte_idx / 4;
    let shift = (byte_idx % 4) * 8;

    let value = (layer[array_idx] >> shift) & 0xFF;
    return f32(value) / 255.0;
}

fn index_ann(x: u32, y: u32) -> u32 {
    let byte_idx = (y * context.dimensions.x + x);
    let array_idx = byte_idx / 4;
    let shift = (byte_idx % 4) * 8;

    let value = (annotations[array_idx] >> shift) & 0xFF;
    return value;
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4<f32> {
    let aspect = context.aspect * f32(context.dimensions.y) / f32(context.dimensions.x);
    let pos = vec2(
        in.position.x * context.scale * aspect,
        in.position.y * context.scale
    ) / 2.0 + 0.5;

    let tex_x_f = pos.x * f32(context.dimensions.x) + context.offset.x;
    let tex_y_f = pos.y * f32(context.dimensions.y) + context.offset.y;

    // Clamp coordinates to prevent out-of-bounds access, which causes a GPU crash.
    let tex_x = u32(clamp(tex_x_f, 0.0, f32(context.dimensions.x - 1u)));
    let tex_y = u32(clamp(tex_y_f, 0.0, f32(context.dimensions.y - 1u)));
    let tex_coord = vec2(f32(tex_x), f32(tex_y));

    // show background differently to highlight the borders
    var value = 0.35;
    // Only sample the texture if the coordinate is within the original viewport
    if (tex_x_f >= 0.0 && tex_x_f < f32(context.dimensions.x) && tex_y_f >= 0.0 && tex_y_f < f32(context.dimensions.y)) {
        value = index(tex_x, tex_y);
    }

    var output_color = vec4<f32>(value, value, value, 1.0);

    if (tex_x_f >= 0.0 && tex_x_f < f32(context.dimensions.x) && tex_y_f >= 0.0 && tex_y_f < f32(context.dimensions.y)) {
        let ann = index_ann(tex_x, tex_y);
	if ((ann & context.show_hide & ANN_DEBUG) > 0) {
	    output_color = COLOR_DEBUG;
	}
	if ((ann & context.show_hide & ANN_INFO) > 0) {
	    output_color = COLOR_INFO;
	}
	if ((ann & context.show_hide & ANN_WARN) > 0) {
	    output_color = COLOR_WARN;
	}
	if ((ann & context.show_hide & ANN_ERROR) > 0) {
	    output_color = COLOR_ERROR;
	}
    }

    return output_color;
}
