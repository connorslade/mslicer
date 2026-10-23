@group(0) @binding(0) var<uniform> ctx: Context;
@group(0) @binding(1) var texture: texture_2d<f32>;
@group(0) @binding(2) var depth: texture_depth_2d;
@group(0) @binding(3) var<storage, read_write> accumulation: array<u32>;
@group(0) @binding(4) var texture_sampler: sampler;

struct Context {
    size: vec2u,
    frame: u32,
    dirty: u32
}

struct FragmentOutput {
    @location(0) color: vec4f,
    @builtin(frag_depth) depth: f32,
}

@fragment
fn frag(in: VertexOutput) -> FragmentOutput {
    let uv = clip_to_uv(in.position);
    var color = textureSample(texture, texture_sampler, uv);
    let depth = textureSample(depth, texture_sampler, uv);

    let px = min(vec2u(uv * vec2f(ctx.size)), ctx.size - 1u);
    let idx = ctx.size.x * px.y + px.x;

    if ctx.dirty == 0 {
        let accm = unpack4x8unorm(accumulation[idx]);
        color = mix(color, accm, 0.9); // dont use 0.5!!
    }

    accumulation[idx] = pack4x8unorm(color);
    return FragmentOutput(color, select(depth, 1.0, depth == 0.0));
}
