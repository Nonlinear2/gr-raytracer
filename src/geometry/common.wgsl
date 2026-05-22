const EULER_STEP_SIZE: f32 = 0.01;
const PI: f32 = 3.141592653589793;
const TAU: f32 = 6.283185307179586;
const MAX_STEPS: u32 = 1000u;

const STOP_MAX_STEPS_REACHED: u32 = 0u;
const STOP_BACKGROUND_REACHED: u32 = 1u;
const STOP_OBJECT_HIT: u32 = 2u;
const STOP_HORIZON_HIT: u32 = 3u;

const SPHERICAL_Z: u32 = 2u;
const SPHERICAL_X: u32 = 3u;

struct Point4 {
    data: vec4<f32>,
    chart: u32,
}

struct FourVector {
    data: vec4<f32>,
    space: u32,
}

struct Photon4 {
    pos: Point4,
    vel: FourVector,
}

struct RayResult {
    photon: Photon4,
    stop_reason: u32,
    padding: vec3<u32>,
}

fn point4_from_components(t: f32, r: f32, theta: f32, phi: f32, chart: u32) -> Point4 {
    return Point4(vec4<f32>(t, r, theta, phi), chart);
}

fn four_vector_from_components(t: f32, r: f32, theta: f32, phi: f32, space: u32) -> FourVector {
    return FourVector(vec4<f32>(t, r, theta, phi), space);
}

fn four_vector_zero(space: u32) -> FourVector {
    return FourVector(vec4<f32>(0.0), space);
}

fn four_vector_as_point4(v: FourVector) -> Point4 {
    return Point4(v.data, v.space);
}

fn wrap_tau(phi: f32) -> f32 {
    return phi - TAU * floor(phi / TAU);
}

fn is_spherical(chart: u32) -> bool {
    return chart == SPHERICAL_X || chart == SPHERICAL_Z;
}

fn zero_matrix() -> mat4x4<f32> {
    return mat4x4<f32>(
        vec4<f32>(0.0),
        vec4<f32>(0.0),
        vec4<f32>(0.0),
        vec4<f32>(0.0)
    );
}

fn quadratic_positive_root(a: f32, b: f32, c: f32) -> f32 {
    let discriminant = max(b * b - 4.0 * a * c, 0.0);
    let sqrt_discriminant = sqrt(discriminant);
    let sign_b = select(1.0, -1.0, b < 0.0);
    let q = -0.5 * (b + sign_b * sqrt_discriminant);
    let root0 = q / a;
    let root1 = c / q;
    return max(root0, root1);
}

fn euler_step(x: Point4, k: FourVector, del_x: Point4, del_k: FourVector) -> Photon4 {
    var new_x = point4_from_components(
        x.data.x + EULER_STEP_SIZE * del_x.data.x,
        x.data.y + EULER_STEP_SIZE * del_x.data.y,
        x.data.z + EULER_STEP_SIZE * del_x.data.z,
        x.data.w + EULER_STEP_SIZE * del_x.data.w,
        x.chart
    );

    var new_k = four_vector_from_components(
        k.data.x + EULER_STEP_SIZE * del_k.data.x,
        k.data.y + EULER_STEP_SIZE * del_k.data.y,
        k.data.z + EULER_STEP_SIZE * del_k.data.z,
        k.data.w + EULER_STEP_SIZE * del_k.data.w,
        k.space
    );

    if (is_spherical(x.chart)) {
        var theta = new_x.data.z;
        var phi = new_x.data.w;
        var k_theta = new_k.data.z;

        if (new_x.data.y < 0.0) {
            new_x.data.y = 0.0;
        }

        if (theta < 0.0) {
            theta = -theta;
            k_theta = -k_theta;
            phi = phi + PI;
        }

        if (theta > PI) {
            theta = TAU - theta;
            k_theta = -k_theta;
            phi = phi + PI;
        }

        new_x.data.z = clamp(theta, 0.0, PI);
        new_x.data.w = wrap_tau(phi);
        new_k.data.z = k_theta;
    }

    return Photon4(new_x, new_k);
}
