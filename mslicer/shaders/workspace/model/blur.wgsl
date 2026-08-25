// Refrence: https://www.shadertoy.com/view/4dfGDH

@group(0) @binding(0) var<uniform> ctx: Context;
@group(0) @binding(1) var normal: texture_2d<f32>;
@group(0) @binding(2) var depth: texture_depth_2d;
@group(0) @binding(3) var occlusion: texture_2d<f32>;
@group(0) @binding(4) var texture_sampler: sampler;

struct Context {
    resolution: vec2u,
    radius: u32,
    σ_spatial: f32,
    σ_depth: f32,
    σ_normal: f32,
}

@fragment
fn frag(in: VertexOutput) -> @location(0) f32 {
    let texel_size = 1.0 / vec2f(ctx.resolution);
    let radius = i32(ctx.radius);

    let uv = clip_to_uv(in.position);
    let this_depth = textureSample(depth, texture_sampler, uv);
    let this_normal = textureSample(normal, texture_sampler, uv).xyz;

    let spatial_norm = 1.0 / normpdf(0.0, ctx.σ_spatial);
    let depth_norm   = 1.0 / normpdf(0.0, ctx.σ_depth);

    var sum = 0.0;
    var total_weight = 0.0;
    for (var y = -radius; y <= radius; y++) {
        for (var x = -radius; x <= radius; x++) {
            let pos = vec2f(vec2(x, y));
            let sample_uv = uv + pos * texel_size;

            let sample_occlusion = textureSample(occlusion, texture_sampler, sample_uv).r;
            let sample_depth = textureSample(depth, texture_sampler, sample_uv);
            let sample_normal = textureSample(normal, texture_sampler, sample_uv).xyz;

            let spatial_weight = normpdf(pos.x, ctx.σ_spatial) * normpdf(pos.y, ctx.σ_spatial) * spatial_norm * spatial_norm;
            let depth_weight = normpdf(sample_depth - this_depth, ctx.σ_depth) * depth_norm;
            let normal_weight = normpdf(1.0 - max(dot(sample_normal, this_normal), 0.0), ctx.σ_normal);

            let weight = spatial_weight * depth_weight * normal_weight;
            sum += sample_occlusion * weight;
            total_weight += weight;
        }
    }

    return sum / total_weight;
}

fn normpdf(x: f32, sigma: f32) -> f32 {
    return 0.39894 * exp(-0.5 * x * x / (sigma * sigma)) / sigma;
}
