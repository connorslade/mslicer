@group(0) @binding(0) var<uniform> ctx: Context;
@group(0) @binding(1) var world: texture_2d<f32>;
@group(0) @binding(2) var depth: texture_depth_2d;
@group(0) @binding(3) var texture_sampler: sampler;

struct Context {
    view: mat4x4f, // world space to clip space
    samples: u32,
    range: f32,
    bias: f32,
}

@fragment
fn frag(in: VertexOutput) -> @location(0) f32 {
    let uv = clip_to_uv(in.position);
    let this_depth = textureSample(depth, texture_sampler, uv);

    if ctx.samples == 0 || this_depth == 1.0 { return 1.0; }

    let world_pos = textureSample(world, texture_sampler, uv).xyz;
    let world_normal = screen_normal(world_pos);

    let pos_u = bitcast<vec3u>(world_pos);
    seed = pos_u.x ^ pos_u.y ^ pos_u.z;

    var occluded = 0u;
    for (var i = 0u; i < ctx.samples; i++) {
        var offset = vec3(rand(), rand(), rand()) * 2.0 - vec3(1.0);
        offset *= sign(dot(offset, world_normal));
        let pos = world_pos + offset * ctx.range;

        let clip = ctx.view * vec4(pos, 1.0);
        let sample_uv = clip_to_uv(clip.xy / clip.w);
        let sample_depth = clip.z / clip.w;

        occluded += u32(textureSample(depth, texture_sampler, sample_uv) < sample_depth - ctx.bias);
    }

    return (1.0 - f32(occluded) / f32(ctx.samples));
}
