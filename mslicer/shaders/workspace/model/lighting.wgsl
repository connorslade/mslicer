@group(0) @binding(0) var<uniform> ctx: Context;
@group(0) @binding(1) var texture: texture_2d<f32>;
@group(0) @binding(2) var normal: texture_2d<f32>;
@group(0) @binding(3) var occlusion: texture_2d<f32>;
@group(0) @binding(4) var texture_sampler: sampler;

struct Context {
    camera_position: vec3f,
    flags: u32
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4f {
    let uv = clip_to_uv(in.position);
    let color = textureSample(texture, texture_sampler, uv);
    let normal = textureSample(normal, texture_sampler, uv).xyz;

    let intensity = blinn_phong(normal, normalize(ctx.camera_position));
    let occlusion = textureSample(occlusion, texture_sampler, uv).r;

    let ao = select(1.0, occlusion, (ctx.flags & 1) != 0);
    return vec4(color.rgb * intensity * ao, color.w);
}
