// Refrence: https://www.shadertoy.com/view/4tf3D8
// todo: replace with morphological antialiasing or smth

@group(0) @binding(0) var<uniform> ctx: Context;
@group(0) @binding(1) var texture: texture_2d<f32>;
@group(0) @binding(2) var texture_sampler: sampler;

struct Context {
    resolution: vec2u,
}

const FXAA_SPAN_MAX: f32 = 8.0;
const FXAA_REDUCE_MUL: f32 = 1.0 / 8.0;
const FXAA_REDUCE_MIN: f32 = 1.0 / 128.0;
const LUMA: vec3f = vec3f(0.299, 0.587, 0.114);

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4f {
    let texel_size = 1.0 / vec2f(ctx.resolution);
    let uv = clip_to_uv(in.position);

    // Read neighbor color values
    let c_nw = textureSample(texture, texture_sampler, uv + vec2f(-1, -1) * texel_size).rgb;
    let c_ne = textureSample(texture, texture_sampler, uv + vec2f( 1, -1) * texel_size).rgb;
    let c_sw = textureSample(texture, texture_sampler, uv + vec2f(-1,  1) * texel_size).rgb;
    let c_se = textureSample(texture, texture_sampler, uv + vec2f( 1,  1) * texel_size).rgb;
    let c_m  = textureSample(texture, texture_sampler, uv).rgb;

    // Convert to lumanance
    let l_nw = dot(c_nw, LUMA);
    let l_ne = dot(c_ne, LUMA);
    let l_sw = dot(c_sw, LUMA);
    let l_se = dot(c_se, LUMA);
    let l_m  = dot(c_m , LUMA);

    var dir = vec2f(-((l_nw + l_ne) - (l_sw + l_se)), (l_nw + l_sw) - (l_ne + l_se));
    let sum = l_nw + l_ne + l_sw + l_se;
    let dir_reduce = max(sum * (0.25 * FXAA_REDUCE_MUL), FXAA_REDUCE_MIN);
    let rcp_dir_min = 1.0 / (min(abs(dir.x), abs(dir.y)) + dir_reduce);

    dir = min(vec2(FXAA_SPAN_MAX), max(vec2(-FXAA_SPAN_MAX), dir * rcp_dir_min)) * texel_size;

    let c_a = 0.5 * (
        textureSample(texture, texture_sampler, uv + dir * (1.0 / 3.0 - 0.5)).rgb +
        textureSample(texture, texture_sampler, uv + dir * (2.0 / 3.0 - 0.5)).rgb
    );
    let c_b = c_a * 0.5 + 0.25 * (
        textureSample(texture, texture_sampler, uv + dir * (0.0 / 3.0 - 0.5)).rgb +
        textureSample(texture, texture_sampler, uv + dir * (3.0 / 3.0 - 0.5)).rgb
    );

    let l_b = dot(c_b, LUMA);
    let l_min = min(l_m, min(min(l_nw, l_ne), min(l_sw, l_se)));
    let l_max = max(l_m, max(max(l_nw, l_ne), max(l_sw, l_se)));

    let color = select(c_a, c_b, l_b >= l_min && l_b <= l_max);

    return vec4f(color, 1.0);
}
