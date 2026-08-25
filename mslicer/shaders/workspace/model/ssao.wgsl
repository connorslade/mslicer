@group(0) @binding(0) var<uniform> ctx: Context;
@group(0) @binding(1) var world: texture_2d<f32>;
@group(0) @binding(2) var depth: texture_depth_2d;
@group(0) @binding(3) var texture_sampler: sampler;

struct Context {
    view: mat4x4f, // world space to clip space
    samples: u32,
    random: u32,
    range: f32,
}

@fragment
fn frag(in: VertexOutput) -> @location(0) f32 {
    let uv = clip_to_uv(in.position);
    let depth = sample_depth(uv);

    if ctx.samples == 0 || depth == 1.0 { return 1.0; }

    let world_pos = textureSample(world, texture_sampler, uv).xyz;
    let world_normal = screen_normal(world_pos);

    seed = u32(abs(i32(world_pos.x * 423123.0) + i32(world_pos.y * 1230.0) + i32(world_pos.z * 12308.0))) + ctx.random;

    var occluded = 0u;
    for (var i = 0u; i < ctx.samples; i++) {
        let offset = vec3(rand(), rand(), rand()) * 2.0 - vec3(1.0);
        let pos = world_pos + offset * ctx.range;

        let clip = ctx.view * vec4(pos, 1.0);
        let uv = clip_to_uv(clip.xy / clip.w);

        occluded += u32(sample_depth(uv) < depth);
    }

    return (1.0 - f32(occluded) / f32(ctx.samples - 1)) * 1.5;
}

fn sample_depth(uv: vec2f) -> f32 {
    return textureLoad(depth, vec2i(uv * vec2f(textureDimensions(depth))), 0);
}
