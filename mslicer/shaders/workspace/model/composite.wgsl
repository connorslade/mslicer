@group(0) @binding(0) var<uniform> ctx: Context;
@group(0) @binding(1) var texture: texture_2d<f32>;
@group(0) @binding(2) var depth: texture_depth_2d;
@group(0) @binding(3) var<storage, read_write> accumulation: array<u32>;
@group(0) @binding(4) var texture_sampler: sampler;

struct Context {
    size: vec2u
}

struct FragmentOutput {
    @location(0) color: vec4f,
    @builtin(frag_depth) depth: f32,
}

@fragment
fn frag(in: VertexOutput) -> FragmentOutput {
    let uv = clip_to_uv(in.position);
    let render = textureSample(texture, texture_sampler, uv);
    let depth = textureSample(depth, texture_sampler, uv);

    let px = vec2u(in.frag_coord.xy);
    let idx = ctx.size.x * px.y + px.x;
    let accm = unpack4x8unorm(accumulation[idx]);

    let color = mix(render, accm, 0.9); // dont use 0.5!!
    accumulation[idx] = pack4x8unorm(color);

    return FragmentOutput(color, select(depth, 1.0, depth == 0.0));
}
