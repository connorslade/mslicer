@group(0) @binding(0) var texture_sampler: sampler;
@group(0) @binding(1) var solid: texture_2d<f32>;
// @group(0) @binding(1) var translucent: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(1) uv: vec2f
}

@vertex
fn vert(@builtin(vertex_index) index: u32) -> VertexOutput {
    let positions = array(
        vec2(-1.0, -1.0), vec2( 1.0, -1.0),
        vec2( 1.0,  1.0), vec2(-1.0,  1.0)
    );
    let uvs = array(
        vec2(0.0, 0.0), vec2(1.0, 0.0),
        vec2(1.0, 1.0), vec2(0.0, 1.0)
    );
    return VertexOutput(vec4(positions[index], 0.0, 1.0), uvs[index]);
}

@fragment
fn frag(in: VertexOutput) -> @location(0) vec4f {
    // let uv = (in.position.xy + vec2(1.0)) / 2.0;
    return textureSample(solid, texture_sampler, in.uv);
}
