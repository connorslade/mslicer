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
