@group(0) @binding(0) var<uniform> ctx: Context;
@group(0) @binding(1) var texture: texture_2d<f32>;
@group(0) @binding(2) var world: texture_multisampled_2d<f32>;
@group(0) @binding(3) var depth: texture_depth_multisampled_2d;
@group(0) @binding(4) var texture_sampler: sampler;

struct Context {
    view: mat4x4f, // world space to clip space
    samples: u32,
    random: u32,
    range: f32,
}

struct VertexOutput {
    @builtin(position) camera_position: vec4f,
    @location(0) position: vec2f,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

@vertex
fn vert(@builtin(vertex_index) index: u32) -> VertexOutput {
    let position = CLIP_TRI[index];
    return VertexOutput(vec4f(vec4(position, 0, 1)), vec2f(position));
}

@fragment
fn frag(in: VertexOutput) -> FragmentOutput {
    let uv = clip_to_uv(in.position);
    let color = textureSample(texture, texture_sampler, uv);
    let depth = sample_depth(uv);

    if ctx.samples == 0 { return FragmentOutput(vec4(color.rgb, color.w), depth); }

    let world_pos = sample_world(uv);
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

    let ao = (1.0 - f32(occluded) / f32(ctx.samples - 1)) * 1.5;
    return FragmentOutput(vec4(color.rgb * ao, color.w), depth);
}

fn clip_to_uv(clip: vec2f) -> vec2f {
    return vec2(0.0, 1.0) + (clip * 0.5 + vec2(0.5)) * vec2f(1.0, -1.0);
}

fn sample_depth(uv: vec2f) -> f32   { return textureLoad(depth,   vec2i(uv * vec2f(textureDimensions(depth)))  , 0);     }
fn sample_world(uv: vec2f) -> vec3f { return textureLoad(world,   vec2i(uv * vec2f(textureDimensions(world)))  , 0).xyz; }
