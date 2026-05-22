struct PackedPoint4 {
    inner: vec4<f32>,
    chart: u32,
}

struct PackedFourVector {
    inner: vec4<f32>,
    vector_space: u32,
}

struct PackedPhoton4 {
    pos: PackedPoint4,
    vel: PackedFourVector,
}

struct PackedRayResult {
    photon: PackedPhoton4,
    stop_reason: u32,
    padding: vec3<u32>,
}

fn new_point4(t: f32, r: f32, theta: f32, phi: f32, chart: u32) -> PackedPoint4 {
    return PackedPoint4(vec4<f32>(t, r, theta, phi), chart);
}

fn new_four_vector(t: f32, r: f32, theta: f32, phi: f32, vector_space: u32) -> PackedFourVector {
    return PackedFourVector(vec4<f32>(t, r, theta, phi), vector_space);
}

fn four_vector_zero(vector_space: u32) -> PackedFourVector {
    return PackedFourVector(vec4<f32>(0.0), vector_space);
}

fn four_vector_as_point4(v: PackedFourVector) -> PackedPoint4 {
    return PackedPoint4(v.inner, v.vector_space);
}