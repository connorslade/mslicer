struct VertexOutput {
    @builtin(position) frag_coord: vec4f,
    @location(0) position: vec2f,
}

@vertex
fn vert(@builtin(vertex_index) index: u32) -> VertexOutput {
    let position = CLIP_TRI[index];
    return VertexOutput(vec4f(vec4(position, 0, 1)), vec2f(position));
}

fn clip_to_uv(clip: vec2f) -> vec2f {
    return vec2(0.0, 1.0) + (clip * 0.5 + vec2(0.5)) * vec2f(1.0, -1.0);
}

fn clip_to_world(inv_view: mat4x4f, clip: vec2f, depth: f32) -> vec3f {
    let world_pos = (inv_view * vec4(clip, depth, 1.0));
    return world_pos.xyz / world_pos.w;
}
