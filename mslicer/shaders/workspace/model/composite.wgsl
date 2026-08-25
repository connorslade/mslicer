@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var depth: texture_depth_2d;
@group(0) @binding(2) var texture_sampler: sampler;

struct FragmentOutput {
    @location(0) color: vec4f,
    @builtin(frag_depth) depth: f32,
}

@fragment
fn frag(in: VertexOutput) -> FragmentOutput {
    let uv = clip_to_uv(in.position);
    let color = textureSample(texture, texture_sampler, uv);
    let depth = textureSample(depth, texture_sampler, uv);

    return FragmentOutput(color, select(depth, 1.0, depth == 0.0));
}
