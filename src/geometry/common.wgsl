const EULER_STEP_SIZE: f32 = 0.01;
const PI: f32 = 3.141592653589793;
const TAU: f32 = 6.283185307179586;
const MAX_STEPS: u32 = 1000u;

// StopReason
const MAX_STEPS_REACHED: u32 = 0u;
const BACKGROUND_REACHED: u32 = 1u;
const OBJECT_HIT: u32 = 2u;
const HORIZON_HIT: u32 = 3u;

// Chart
const CARTESIAN_WORLD: u32 = 0u;
const CARTESIAN: u32 = 1u;
const SPHERICAL_Z: u32 = 2u;
const SPHERICAL_X: u32 = 3u;

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
