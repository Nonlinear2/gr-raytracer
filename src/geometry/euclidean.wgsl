fn g(pos: Point4) -> mat4x4<f32> {
    return mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );
}

fn g_inv(pos: Point4) -> mat4x4<f32> {
    return mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );
}

fn del_g(pos: Point4, i: u32) -> mat4x4<f32> {
    return zero_matrix();
}

fn preferred_chart_for_point(world_pos: vec3<f32>) -> u32 { // world_pos must be in CARTESIAN_WORLD chart
    return CHART_CARTESIAN;
}

fn is_close_to_singular(pos: Point4) -> bool {
    return false;
}