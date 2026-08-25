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

    let σ_spatial_sq = ctx.σ_spatial * ctx.σ_spatial;
    let σ_depth_sq = ctx.σ_depth * ctx.σ_depth;
    let σ_normal_sq = ctx.σ_normal * ctx.σ_normal;

    let uv = clip_to_uv(in.position);
    let this_depth = textureSample(depth, texture_sampler, uv);
    let this_normal = textureSample(normal, texture_sampler, uv).xyz;

    var sum = 0.0;
    var total_weight = 0.0;
    for (var y = -radius; y <= radius; y = y + 1) {
        for (var x = -radius; x <= radius; x = x + 1) {
            let pos = vec2f(vec2(x, y));
            let offset = pos * texel_size;
            let sample_uv = uv + offset;

            let sample_occlusion = textureSample(occlusion, texture_sampler, sample_uv).r;
            let sample_depth = textureSample(depth, texture_sampler, sample_uv);
            let sample_normal = textureSample(normal, texture_sampler, sample_uv).xyz;

            let spatial_dist_sq = length(pos);
            let spatial_weight = exp(-spatial_dist_sq / (2.0 * σ_spatial_sq));

            let depth_diff = sample_depth - this_depth;
            let depth_weight = exp(-(depth_diff * depth_diff) / (2.0 * σ_depth_sq));

            let normal_diff = max(dot(sample_normal, this_normal), 0.0);
            let normal_weight = pow(normal_diff, 1.0 / σ_normal_sq);

            let weight = spatial_weight * depth_weight * normal_weight;
            sum += sample_occlusion * weight;
            total_weight += weight;
        }
    }

    return sum / total_weight;
}
