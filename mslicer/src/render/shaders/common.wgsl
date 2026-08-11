const CLIP_TRI = array(
    vec2( 1.0,  1.0),
    vec2(-3.0,  1.0),
    vec2( 1.0, -3.0)
);

fn screen_normal(world_position: vec3f) -> vec3f {
    let dy = dpdy(world_position);
    let dx = dpdx(world_position);
    return normalize(cross(dy, dx));
}

fn blinn_phong(normal: vec3f, light: vec3f) -> f32 {
    let diffuse = max(dot(normal, light), 0.0);
    let reflect_dir = reflect(-light, normal);
    let specular = pow(max(dot(light, reflect_dir), 0.0), 32.0);

    return diffuse + specular + 0.1;
}

var<private> seed: u32 = 0u;

fn rand() -> f32 {
    seed = seed * 747796405u + 2891336453u;
    let f = f32(seed >> 9u) / f32(1u << 23u);
    return fract(f);
}

fn rand_full() -> f32 {
    return rand() * 2.0 - 1.0;
}
