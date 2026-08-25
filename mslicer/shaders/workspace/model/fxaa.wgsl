@group(0) @binding(0) var<uniform> ctx: Context;
@group(0) @binding(1) var texture: texture_2d<f32>;
@group(0) @binding(2) var texture_sampler: sampler;

struct Context {
    resolution: vec2u,
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4f {
    let uv = clip_to_uv(in.position);
    let color = textureSample(texture, texture_sampler, uv);

    // todo: fxaa…

    return color;
}
