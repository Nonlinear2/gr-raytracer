// common.wgsl

const EULER_STEP_SIZE: f32 = 0.005;
const RK4_STEP_SIZE: f32 = 0.005;

const PI: f32 = 3.141592653589793;
const TAU: f32 = 6.283185307179586;

const MAX_STEPS: u32 = 0u; // filled by get_shader
const DEBUG_RAY_TRAJECTORY: u32 = 0u; // filled by get_shader

const EPS: f32 = 10e-10;

// StopReason
const STOP_MAX_STEPS_REACHED: u32 = 0u;
const STOP_BACKGROUND_REACHED: u32 = 1u;
const STOP_OBJECT_HIT: u32 = 2u;
const STOP_HORIZON_HIT: u32 = 3u;

/// which global chart we use to describe points on the submanifolds of R^4 obtained by fixing the time coordinate.
/// Important points: 
/// These charts will designate the maps from coordinates to "manifold" and not the opposite. They are technically inverse charts

// Chart
const CHART_CARTESIAN_WORLD: u32 = 0u;
const CHART_CARTESIAN: u32 = 1u; // cartesian with center point
const CHART_SPHERICAL_Z: u32 = 2u; // spherical coordinates with center and (r: 1, theta: 0, phi: ...) pointing towards positive Z 
const CHART_SPHERICAL_X: u32 = 3u; // spherical coordinates with center and (r: 1, theta: 0, phi: ...) pointing towards positive X 

// TangentSpace
const TANGENT_CARTESIAN_WORLD: u32 = 0u;
const TANGENT_CARTESIAN: u32 = 1u;
const TANGENT_SPHERICAL_Z: u32 = 2u;
const TANGENT_SPHERICAL_X: u32 = 3u;

// objects
const OBJECT_NONE: u32 = 0u;
const OBJECT_SPHERE: u32 = 1u;
const OBJECT_DISC: u32 = 2u;

// materials
const MATERIAL_NONE: u32 = 0u;
const MATERIAL_DIFFUSE: u32 = 1u;
const MATERIAL_METAL: u32 = 2u;

const MAX_BOUNCES: u32 = 2u;

const VEC3_ZERO: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
const VEC4_ZERO: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);

fn zero_matrix() -> mat4x4<f32> {
    return mat4x4<f32>(
        vec4<f32>(0.0),
        vec4<f32>(0.0),
        vec4<f32>(0.0),
        vec4<f32>(0.0)
    );
}

fn quadratic_positive_root(a: f32, b: f32, c: f32) -> f32 {
    let delta = max(b * b - 4.0 * a * c, 0.0);
    let sign_b = select(1.0, -1.0, b < 0.0);
    let q = -0.5 * (b + sign_b * sqrt(delta));
    let root0 = q / a;
    let root1 = c / q;
    return max(root0, root1);
}

fn hash_u32(x: u32) -> u32 {
    var h = x;
    h = h ^ 61u ^ (h >> 16u);
    h = h * 9u;
    h = h ^ (h >> 4u);
    h = h * 0x27d4eb2du;
    h = h ^ (h >> 15u);
    return h;
}

fn rand(seed: u32) -> f32 {
    return f32(hash_u32(seed)) / 4294967295.0;
}

fn random_unit_vector(seed0: u32, seed1: u32) -> vec3<f32> {
    let z = 2.0 * rand(seed0) - 1.0;
    let phi = TAU * rand(seed1);
    let r_xy = sqrt(max(0.0, 1.0 - z * z));
    return vec3<f32>(r_xy * cos(phi), r_xy * sin(phi), z);
}

fn sky_color(world_pos: vec3<f32>) -> vec3<f32> {
    let tx = floor(world_pos.x * 2.0);
    let ty = floor(world_pos.y * 2.0);
    if (u32(abs(i32(tx + ty))) % 2u == 0u) {
        return vec3<f32>(35.0 / 255.0, 105.0 / 255.0, 105.0 / 255.0);
    }
    return vec3<f32>(235.0 / 255.0);
}

// packed_types.wgsl

struct Point3 {
    inner: vec3<f32>,
    chart: u32,
}

struct Point4 {
    inner: vec4<f32>,
    chart: u32,
}

struct ThreeVector {
    inner: vec3<f32>,
    vector_space: u32,
}

struct FourVector {
    inner: vec4<f32>,
    vector_space: u32,
}

struct Photon3 {
    pos: Point3,
    vel: ThreeVector,
}

struct Photon4 {
    pos: Point4,
    vel: FourVector,
}

struct PhotonDerivative {
    d_pos: Point4,
    d_vel: FourVector,
}

struct ColorResult {
    color: vec3<f32>,
    _pad0: f32,
}

fn packed_color_result(color: vec3<f32>) -> ColorResult {
    return ColorResult(color, 0.0);
}

struct TracePos {
    pos: vec3<f32>,
    fill_flag: f32,
}

struct TraceResult {
    positions: array<TracePos, 1000u>,
}

fn new_point3(r: f32, theta: f32, phi: f32, chart: u32) -> Point3 {
    return Point3(vec3<f32>(r, theta, phi), chart);
}

fn new_three_vector(r: f32, theta: f32, phi: f32, vector_space: u32) -> ThreeVector {
    return ThreeVector(vec3<f32>(r, theta, phi), vector_space);
}

fn new_point4(t: f32, r: f32, theta: f32, phi: f32, chart: u32) -> Point4 {
    return Point4(vec4<f32>(t, r, theta, phi), chart);
}

fn new_four_vector(t: f32, r: f32, theta: f32, phi: f32, vector_space: u32) -> FourVector {
    return FourVector(vec4<f32>(t, r, theta, phi), vector_space);
}

fn four_vector_zero(vector_space: u32) -> FourVector {
    return FourVector(vec4<f32>(0.0), vector_space);
}

fn four_vector_as_point4(v: FourVector) -> Point4 {
    return Point4(v.inner, v.vector_space);
}

fn reflect(v: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    return v - 2.0 * dot(v, n) * n;
}


struct RayTraceState {
    ray: Photon4,
    bounce_count: u32,
    radiance: vec3<f32>,
    throughput: vec3<f32>,
}

fn new_ray_trace_state(ray: Photon4) -> RayTraceState {
    return RayTraceState(ray, 0u, VEC3_ZERO, vec3<f32>(1.0, 1.0, 1.0));
}

const RAY_TRACE_STATE_ZERO = RayTraceState(
    Photon4(Point4(VEC4_ZERO, CHART_CARTESIAN_WORLD), FourVector(VEC4_ZERO, TANGENT_CARTESIAN_WORLD)), 
    0u, VEC3_ZERO, vec3<f32>(1.0, 1.0, 1.0)
);

// surface.wgsl

struct Material {
    kind: u32,
    _pad0: array<u32, 3>,
    color: vec3<f32>,
    params: f32,
    _pad1: array<u32, 4>,
}

//          |               Diffuse              |        Metal
// params:  | None                               | fuzz


struct PackedObject {
    kind: u32,
    material: Material,
    _pad0: array<u32, 3>,
    data0: vec4<f32>,
    data1: vec4<f32>,
    emission_params: vec4<f32>,
}

//          |               sphere              |        disc
// data0.x: | center.x (chart: CARTESIAN_WORLD) | center.x (chart: CARTESIAN_WORLD)
// data0.y: | center.y (chart: CARTESIAN_WORLD) | center.y (chart: CARTESIAN_WORLD)
// data0.z: | center.z (chart: CARTESIAN_WORLD) | center.z (chart: CARTESIAN_WORLD)
// data0.w: | radius                            | radius
// data1.x: | None                              | normal.x (tangent space: CARTESIAN_WORLD)
// data1.y: | None                              | normal.y (tangent space: CARTESIAN_WORLD)
// data1.z: | None                              | normal.z (tangent space: CARTESIAN_WORLD)
// data1.w: | None                              | None

// if is_hit is false, the other values dont have meaning
struct HitData {
    is_hit: u32,
    hit_point: vec3<f32>, // in CARTESIAN_WORLD
    incoming_dir: vec3<f32>, // in TANGENT_CARTESIAN_WORLD
    normal: vec3<f32>, // in TANGENT_CARTESIAN_WORLD
}

const NO_HIT = HitData(0, VEC3_ZERO, VEC3_ZERO, VEC3_ZERO);

// manifold vel is in any chart
fn object_hit(object: PackedObject, prev_pos: vec3<f32>, new_pos: vec3<f32>) -> HitData {
    switch object.kind {
        case OBJECT_SPHERE: { return sphere_hit(object, prev_pos, new_pos); }
        case OBJECT_DISC: { return disc_hit(object, prev_pos, new_pos); }
        default: { return NO_HIT; }
    }
}

// manifold vel is in any chart
// pos is in CARTESIAN_WORLD chart
// returns: hit_pos, is_hit
fn sphere_hit(object: PackedObject, prev_pos: vec3<f32>, new_pos: vec3<f32>) -> HitData {
    let center = object.data0.xyz;
    let radius = object.data0.w;

    let normal = normalize(new_pos - center);
    let hit_point = center + normal * (radius * (1.0 + EPS)); //avoid precision errors

    if (length(new_pos - center) > radius) { // no hit
        return NO_HIT;
    }

    return HitData(
        1,
        hit_point,
        normalize(new_pos - prev_pos),
        normal
    );
}

// manifold vel is in any chart
// prev_pos and new_pos are in CARTESIAN_WORLD chart
// returns: hit_pos, is_hit
fn disc_hit(object: PackedObject, prev_pos: vec3<f32>, new_pos: vec3<f32>) -> HitData {
    let center = object.data0.xyz;
    let radius = object.data0.w;

    let normal = normalize(object.data1.xyz);

    let segment = new_pos - prev_pos;
    let segment_dot_normal = dot(segment, normal);

    if (abs(segment_dot_normal) < EPS) { // movement parallel to disc, no intersection.
        return NO_HIT;
    }

    // we are searching for t such that
    // (X - center) . normal = 0
    // (prev_pos + t*segment - center) . normal = 0
    // (t*segment) . normal = (center - prev_pos) . normal
    // t = ((center - prev_pos) . normal) / (segment . normal)
    let t = dot(center - prev_pos, normal) / segment_dot_normal;

    if (t < 0.0 || t > 1.0) { // check if intersection with disc plane is between prev_pos and new_pos
        return NO_HIT;
    }

    let hit_point = prev_pos + t * segment;
    if (length(hit_point - center) > radius) {
        return NO_HIT;
    }

    let directed_normal = select(normal, -normal, sign(segment_dot_normal) > 0);
    return HitData(
        1,
        hit_point + EPS * directed_normal,
        normalize(new_pos - prev_pos),
        directed_normal
    );
}

fn material_scatter(
    material: Material,
    hit_data: HitData,
    rng_seed: u32
) -> vec4<f32> {

    switch material.kind {
        case MATERIAL_DIFFUSE: { return diffuse_scatter(hit_data, rng_seed); }
        case MATERIAL_METAL: { return metal_scatter(hit_data, rng_seed, material.params); }
        default: { return VEC4_ZERO; }
    }
}


fn diffuse_scatter(hit_data: HitData, rng_seed: u32) -> vec4<f32> {
    let rand_dir = random_unit_vector(rng_seed ^ 0xA341316Cu, rng_seed ^ 0xC8013EA4u);

    let scattered = normalize(hit_data.normal + 0.99 * rand_dir);

    return vec4<f32>(scattered, 1.0);
}

fn metal_scatter(hit_data: HitData, rng_seed: u32, fuzz: f32) -> vec4<f32> {
    let rand_dir = random_unit_vector(rng_seed ^ 0xA341316Cu, rng_seed ^ 0xC8013EA4u);

    let f = clamp(fuzz, 0.0, 0.99);
    var scattered_dir = normalize(reflect(hit_data.incoming_dir, hit_data.normal) + f * rand_dir);

    if (dot(scattered_dir, hit_data.normal) <= 0.0) {
        return VEC4_ZERO;
    }

    return vec4<f32>(scattered_dir, 1.0);
}

// euler.wgsl

fn wrap_photon(pos: Point4, vel: FourVector) -> Photon4 {
    var new_pos = pos;
    var new_vel = vel;

    if (pos.chart == CHART_SPHERICAL_X || pos.chart == CHART_SPHERICAL_Z) {
        var theta = new_pos.inner.z;
        var phi = new_pos.inner.w;
        var k_theta = new_vel.inner.z;

        // a negative radius should never be reached.

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

        new_pos.inner.z = clamp(theta, 0.0, PI);
        new_pos.inner.w = phi - TAU * floor(phi / TAU); // mod tau
        new_vel.inner.z = k_theta;
    }

    return Photon4(new_pos, new_vel);
}

fn euler_step(photon: Photon4) -> Photon4 {
    let derivative = geodesic_derivative(photon);
    let pos = photon.pos;
    let vel = photon.vel;

    var new_pos = Point4(
        pos.inner + EULER_STEP_SIZE * derivative.d_pos.inner,
        pos.chart
    );

    var new_vel = FourVector(
        vel.inner + EULER_STEP_SIZE * derivative.d_vel.inner,
        vel.vector_space
    );

    return wrap_photon(new_pos, new_vel);
}

// runge_kutta.wgsl

fn rk4_step(photon: Photon4) -> Photon4 {
    // k1 = geodesic_derivative(photon)
    // k2 = geodesic_derivative(photon + k1*h/2)
    // k3 = geodesic_derivative(photon + k2*h/2)
    // k4 = geodesic_derivative(photon + k3*h)
    // new_photon = photon + h * (k1 + 2*k2 + 2*k3 + k4) / 6

    let pos = photon.pos;
    let vel = photon.vel;
    let h   = RK4_STEP_SIZE;

    let k1 = geodesic_derivative(photon);

    let k2 = geodesic_derivative(wrap_photon(
        Point4(pos.inner + k1.d_pos.inner * h/2.0, pos.chart),
        FourVector(vel.inner + k1.d_vel.inner * h/2.0, vel.vector_space)
    ));

    let k3 = geodesic_derivative(wrap_photon(
        Point4(pos.inner + k2.d_pos.inner * h/2.0, pos.chart),
        FourVector(vel.inner + k2.d_vel.inner * h/2.0, vel.vector_space)
    ));

    let k4 = geodesic_derivative(wrap_photon(
        Point4(pos.inner + k3.d_pos.inner * h, pos.chart),
        FourVector(vel.inner + k3.d_vel.inner * h, vel.vector_space)
    ));

    let new_pos = Point4(
        pos.inner + h * (k1.d_pos.inner + 2.0*k2.d_pos.inner + 2.0*k3.d_pos.inner + k4.d_pos.inner) / 6.0,
        pos.chart
    );
    let new_vel = FourVector(
        vel.inner + h * (k1.d_vel.inner + 2.0*k2.d_vel.inner + 2.0*k3.d_vel.inner + k4.d_vel.inner) / 6.0,
        vel.vector_space
    );

    return wrap_photon(new_pos, new_vel);
}

// schwarzschild.wgsl

// CONSTS
const SUBATLAS_CENTER: vec3<f32> = vec3<f32>(0.0, 0.0, -1.0);  // center is a point in world space
const R_S: f32 = 0.25;
const SCENE_SIZE: f32 = 3.0;


// HasAtlas3 for Schwarzschild4Manifold
// Atlas describing submanifolds of R^4 given by fixing the time coordinate (so this coordinate doesnt get converted).

fn preferred_chart_for_point(world_pos: vec3<f32>) -> u32 { // world_pos must be in CARTESIAN_WORLD chart
    let p_rel = world_pos - SUBATLAS_CENTER;
    let dist_to_z_axis_sq = p_rel.x * p_rel.x + p_rel.y * p_rel.y;
    let dist_to_x_axis_sq = p_rel.y * p_rel.y + p_rel.z * p_rel.z;

    if (dist_to_z_axis_sq < dist_to_x_axis_sq) {
        return CHART_SPHERICAL_X;
    }
    return CHART_SPHERICAL_Z;
}

fn transition_point(point: Point3, to: u32) -> Point3 {
    if (point.chart == to) {
        return point;
    }

    switch (point.chart) {
        case CHART_CARTESIAN_WORLD: {
            let world_pos = point.inner;
            let p_rel = world_pos - SUBATLAS_CENTER;
            let r = length(p_rel);

            switch (to) {
                case CHART_SPHERICAL_Z: {
                    let new_theta = acos(p_rel.z / r);
                    var new_phi = atan2(p_rel.y, p_rel.x);
                    if (new_phi < 0.0) { new_phi = new_phi + TAU; }
                    return new_point3(r, new_theta, new_phi, CHART_SPHERICAL_Z);
                }
                case CHART_SPHERICAL_X: {
                    let new_theta = acos(p_rel.x / r);
                    var new_phi = atan2(p_rel.z, p_rel.y);
                    if (new_phi < 0.0) { new_phi = new_phi + TAU; }
                    return new_point3(r, new_theta, new_phi, CHART_SPHERICAL_X);
                }
                default: {
                    return point;
                }
            }
        }
        case CHART_SPHERICAL_Z: {
            switch (to) {
                case CHART_CARTESIAN_WORLD: {
                    let r = point.inner.x;
                    let theta = point.inner.y;
                    let phi = point.inner.z;

                    let x = r * sin(theta) * cos(phi);
                    let y = r * sin(theta) * sin(phi);
                    let z = r * cos(theta);
                    return new_point3(x + SUBATLAS_CENTER.x, y + SUBATLAS_CENTER.y, z + SUBATLAS_CENTER.z, CHART_CARTESIAN_WORLD);
                }
                case CHART_SPHERICAL_X: {
                    let r = point.inner.x;
                    let theta = point.inner.y;
                    let phi = point.inner.z;
                    let sin_theta = sin(theta);
                    let cos_theta = cos(theta);
                    let sin_phi = sin(phi);
                    let cos_phi = cos(phi);

                    let new_theta = acos(sin_theta * cos_phi);
                    var new_phi = atan2(cos_theta, sin_theta * sin_phi);
                    if (new_phi < 0.0) { new_phi = new_phi + TAU; }
                    return new_point3(r, new_theta, new_phi, CHART_SPHERICAL_X);
                }
                default: {
                    return point;
                }
            }
        }
        case CHART_SPHERICAL_X: {
            switch (to) {
                case CHART_CARTESIAN_WORLD: {
                    let r = point.inner.x;
                    let theta = point.inner.y;
                    let phi = point.inner.z;

                    let x = r * cos(theta);
                    let y = r * sin(theta) * cos(phi);
                    let z = r * sin(theta) * sin(phi);
                    return new_point3(x + SUBATLAS_CENTER.x, y + SUBATLAS_CENTER.y, z + SUBATLAS_CENTER.z, CHART_CARTESIAN_WORLD);
                }
                case CHART_SPHERICAL_Z: {
                    let r = point.inner.x;
                    let theta = point.inner.y;
                    let phi = point.inner.z;
                    let sin_theta = sin(theta);
                    let cos_theta = cos(theta);
                    let sin_phi = sin(phi);
                    let cos_phi = cos(phi);

                    let new_theta = acos(sin_theta * sin_phi);
                    var new_phi = atan2(sin_theta * cos_phi, cos_theta);
                    if (new_phi < 0.0) { new_phi = new_phi + TAU; }
                    return new_point3(r, new_theta, new_phi, CHART_SPHERICAL_Z);
                }
                default: {
                    return point;
                }
            }
        }
        default: {
            return point;
        }
    }
}

fn transition_vector(point: Point3, v: ThreeVector, to: u32) -> ThreeVector {
    if (point.chart == to) {
        return v;
    }

    let r = point.inner.x;
    let theta = point.inner.y;
    let phi = point.inner.z;

    switch (point.chart) {
        case CHART_CARTESIAN_WORLD: {
            let world_pos = point.inner;
            let p_rel = world_pos - SUBATLAS_CENTER;
            let x = p_rel.x;
            let y = p_rel.y;
            let z = p_rel.z;
            let rho_z = sqrt(x * x + y * y);
            let rho_x = sqrt(y * y + z * z);
            let r = sqrt(x * x + y * y + z * z);

            switch (to) {
                case CHART_SPHERICAL_Z: {
                    let dr_dx = x / r;
                    let dr_dy = y / r;
                    let dr_dz = z / r;
                    let dth_dx = x * z / (r * r * rho_z);
                    let dth_dy = y * z / (r * r * rho_z);
                    let dth_dz = -rho_z / (r * r);
                    let dph_dx = -y / (rho_z * rho_z);
                    let dph_dy = x / (rho_z * rho_z);

                    return new_three_vector(
                        dr_dx * v.inner.x + dr_dy * v.inner.y + dr_dz * v.inner.z,
                        dth_dx * v.inner.x + dth_dy * v.inner.y + dth_dz * v.inner.z,
                        dph_dx * v.inner.x + dph_dy * v.inner.y,
                        TANGENT_SPHERICAL_Z,
                    );
                }
                case CHART_SPHERICAL_X: {
                    let dr_dx = x / r;
                    let dr_dy = y / r;
                    let dr_dz = z / r;
                    let dth_dx = -rho_x / (r * r);
                    let dth_dy = x * y / (r * r * rho_x);
                    let dth_dz = x * z / (r * r * rho_x);
                    let dph_dy = -z / (rho_x * rho_x);
                    let dph_dz = y / (rho_x * rho_x);

                    return new_three_vector(
                        dr_dx * v.inner.x + dr_dy * v.inner.y + dr_dz * v.inner.z,
                        dth_dx * v.inner.x + dth_dy * v.inner.y + dth_dz * v.inner.z,
                        dph_dy * v.inner.y + dph_dz * v.inner.z,
                        TANGENT_SPHERICAL_X,
                    );
                }
                default: {
                    return v;
                }
            }
        }
        case CHART_SPHERICAL_Z: {
            switch (to) {
                case CHART_CARTESIAN_WORLD: {
                    let v_r = v.inner.x;
                    let v_theta = v.inner.y;
                    let v_phi = v.inner.z;
                    let sin_theta = sin(theta);
                    let cos_theta = cos(theta);
                    let sin_phi = sin(phi);
                    let cos_phi = cos(phi);

                    return new_three_vector(
                        sin_theta * cos_phi * v_r + r * cos_theta * cos_phi * v_theta - r * sin_theta * sin_phi * v_phi,
                        sin_theta * sin_phi * v_r + r * cos_theta * sin_phi * v_theta + r * sin_theta * cos_phi * v_phi,
                        cos_theta * v_r - r * sin_theta * v_theta,
                        TANGENT_CARTESIAN_WORLD,
                    );
                }
                case CHART_SPHERICAL_X: {
                    let v_theta = v.inner.y;
                    let v_phi = v.inner.z;
                    let sin_theta = sin(theta);
                    let cos_theta = cos(theta);
                    let sin_phi = sin(phi);
                    let cos_phi = cos(phi);

                    let denom = sqrt(cos_theta * cos_theta + sin_theta * sin_theta * sin_phi * sin_phi);
                    let denom2 = denom * denom;

                    return new_three_vector(
                        v.inner.x,
                        (-cos_theta * cos_phi * v_theta + sin_theta * sin_phi * v_phi) / denom,
                        (-sin_phi * v_theta - cos_theta * sin_theta * cos_phi * v_phi) / denom2,
                        TANGENT_SPHERICAL_X,
                    );
                }
                default: {
                    return v;
                }
            }
        }
        case CHART_SPHERICAL_X: {
            switch (to) {
                case CHART_CARTESIAN_WORLD: {
                    let v_r = v.inner.x;
                    let v_theta = v.inner.y;
                    let v_phi = v.inner.z;
                    let sin_theta = sin(theta);
                    let cos_theta = cos(theta);
                    let sin_phi = sin(phi);
                    let cos_phi = cos(phi);

                    return new_three_vector(
                        cos_theta * v_r - r * sin_theta * v_theta,
                        sin_theta * cos_phi * v_r + r * cos_theta * cos_phi * v_theta - r * sin_theta * sin_phi * v_phi,
                        sin_theta * sin_phi * v_r + r * cos_theta * sin_phi * v_theta + r * sin_theta * cos_phi * v_phi,
                        TANGENT_CARTESIAN_WORLD,
                    );
                }
                case CHART_SPHERICAL_Z: {
                    let v_theta = v.inner.y;
                    let v_phi = v.inner.z;
                    let sin_theta = sin(theta);
                    let cos_theta = cos(theta);
                    let sin_phi = sin(phi);
                    let cos_phi = cos(phi);

                    let denom = sqrt(cos_theta * cos_theta + sin_theta * sin_theta * cos_phi * cos_phi);
                    let denom2 = denom * denom;

                    return new_three_vector(
                        v.inner.x,
                        (-cos_theta * sin_phi * v_theta - sin_theta * cos_phi * v_phi) / denom,
                        (cos_phi * v_theta - cos_theta * sin_theta * sin_phi * v_phi) / denom2,
                        TANGENT_SPHERICAL_Z,
                    );
                }
                default: {
                    return v;
                }
            }
        }
        default: {
            return v;
        }
    }
}

// impl PseudoRiemanian4Manifold for Schwarzschild4Manifold

fn photon3_to_photon4(photon: Photon3, chart: u32) -> Photon4 { // resets time component to 0, only works in static spacetime
    let pos = transition_point(photon.pos, chart);
    let vel = transition_vector(photon.pos, photon.vel, chart);

    let photon_x = new_point4(0.0, pos.inner.x, pos.inner.y, pos.inner.z, chart);

    let g_mat = g(photon_x);
        let b = 2.0 * (g_mat[0][1] * vel.inner.x + g_mat[0][2] * vel.inner.y + g_mat[0][3] * vel.inner.z);
    let c =
                    g_mat[1][1] * vel.inner.x * vel.inner.x
                + 2.0 * g_mat[1][2] * vel.inner.x * vel.inner.y
                + 2.0 * g_mat[1][3] * vel.inner.x * vel.inner.z
                + g_mat[2][2] * vel.inner.y * vel.inner.y
                + 2.0 * g_mat[2][3] * vel.inner.y * vel.inner.z
                + g_mat[3][3] * vel.inner.z * vel.inner.z;

    let k_0 = quadratic_positive_root(g_mat[0][0], b, c);
    return Photon4(photon_x, FourVector(vec4<f32>(k_0, vel.inner.x, vel.inner.y, vel.inner.z), chart));
}

fn transition_photon4(photon: Photon4, chart: u32) -> Photon4 {
    let pos3 = Point3(photon.pos.inner.yzw, photon.pos.chart);
    let vel3 = ThreeVector(photon.vel.inner.yzw, photon.vel.vector_space);
    return photon3_to_photon4(Photon3(pos3, vel3), chart);
}

fn g(pos: Point4) -> mat4x4<f32> {
    let r = pos.inner.y;
    let theta = pos.inner.z;
    let f = 1.0 - R_S / r;

    return mat4x4<f32>(
        vec4<f32>(f, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, -1.0 / f, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, -r * r, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, -r * r * sin(theta) * sin(theta))
    );
}

fn g_inv(pos: Point4) -> mat4x4<f32> {
    let r = pos.inner.y;
    let theta = pos.inner.z;
    let f = 1.0 - R_S / r;

    return mat4x4<f32>(
        vec4<f32>(1.0 / f, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, R_S / r - 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, -1.0 / (r * r), 0.0),
        vec4<f32>(0.0, 0.0, 0.0, -1.0 / (r * r * sin(theta) * sin(theta)))
    );
}

fn del_g(pos: Point4, i: u32) -> mat4x4<f32> {
    let r = pos.inner.y;
    let theta = pos.inner.z;

    if (i == 1u) {
        let f = 1.0 - R_S / r;
        return mat4x4<f32>(
            vec4<f32>(R_S / (r * r), 0.0, 0.0, 0.0),
            vec4<f32>(0.0, R_S / (r * r * f * f), 0.0, 0.0),
            vec4<f32>(0.0, 0.0, -2.0 * r, 0.0),
            vec4<f32>(0.0, 0.0, 0.0, -2.0 * r * sin(theta) * sin(theta))
        );
    }

    if (i == 2u) {
        return mat4x4<f32>(
            vec4<f32>(0.0, 0.0, 0.0, 0.0),
            vec4<f32>(0.0, 0.0, 0.0, 0.0),
            vec4<f32>(0.0, 0.0, 0.0, 0.0),
            vec4<f32>(0.0, 0.0, 0.0, -2.0 * r * r * cos(theta) * sin(theta))
        );
    }

    return zero_matrix();
}

fn christoffel(pos: Point4, mu: u32, nu: u32, lambda: u32) -> f32 {
    let g_inv = g_inv(pos);
    let d_mu_g = del_g(pos, mu);
    let d_nu_g = del_g(pos, nu);

    var gamma = 0.0;
    for (var alpha: u32 = 0u; alpha < 4u; alpha = alpha + 1u) {
        let d_alpha_g = del_g(pos, alpha);
        gamma = gamma + 0.5 * g_inv[lambda][alpha] * (
            d_mu_g[alpha][nu] + d_nu_g[alpha][mu] - d_alpha_g[mu][nu]
        );
    }

    return gamma;
}

fn geodesic_derivative(photon: Photon4) -> PhotonDerivative {
    var del_k = four_vector_zero(photon.vel.vector_space);

    for (var mu: u32 = 0u; mu < 4u; mu = mu + 1u) {
        for (var alpha: u32 = 0u; alpha < 4u; alpha = alpha + 1u) {
            for (var beta: u32 = 0u; beta < 4u; beta = beta + 1u) {
                let gamma = christoffel(photon.pos, alpha, beta, mu);
                del_k.inner[mu] = del_k.inner[mu] - gamma * photon.vel.inner[alpha] * photon.vel.inner[beta];
            }
        }
    }

    return PhotonDerivative(four_vector_as_point4(photon.vel), del_k); 
}

fn step_along_null_geodesic(photon: Photon4) -> Photon4 {
    return rk4_step(photon);
}

fn evolve_ray(input_ray: Photon4, ray_index: u32) -> ColorResult {
    var state = new_ray_trace_state(input_ray);

    for (var step: u32 = 0u; step < MAX_STEPS; step = step + 1u) {
        let prev_ray = state.ray;
        state.ray = step_along_null_geodesic(state.ray);

        let prev_ray_pos3 = Point3(prev_ray.pos.inner.yzw, prev_ray.pos.chart);
        let prev_world_pos = transition_point(prev_ray_pos3, CHART_CARTESIAN_WORLD).inner;

        let ray_pos3 = Point3(state.ray.pos.inner.yzw, state.ray.pos.chart);
        let world_pos = transition_point(ray_pos3, CHART_CARTESIAN_WORLD).inner;

        if (DEBUG_RAY_TRAJECTORY == 1u) {
            trace_results.positions[step] = TracePos(world_pos, 1.0);
        }

        if (state.ray.pos.inner.y <= R_S) { // hit singularity
            return packed_color_result(state.radiance);
        }

        if (length(world_pos - SUBATLAS_CENTER) > SCENE_SIZE) { // background reached 
            return packed_color_result(state.radiance + state.throughput * sky_color(world_pos));
        }

        let preferred_chart = preferred_chart_for_point(world_pos);
        if (preferred_chart != state.ray.pos.chart) {
            state.ray = transition_photon4(state.ray, preferred_chart);
        }

        for (var obj_idx: u32 = 0u; obj_idx < arrayLength(&objects.data); obj_idx = obj_idx + 1u) {
            let object = objects.data[obj_idx];

            let hit_data = object_hit(object, prev_world_pos, world_pos);

            if (hit_data.is_hit == 0) {
                continue;
            }

            // // DEBUG: return magenta for sphere hits so we can detect them uniquely
            // if (object.kind == OBJECT_DISC) {
            //     return packed_color_result(vec3<f32>(1.0, 0.0, 1.0));
            // }

            if (state.bounce_count >= MAX_BOUNCES) {
                return packed_color_result(state.radiance);
            }

            let rng_seed = ray_index * 73856093u + step * 19349663u 
                + obj_idx * 83492791u + state.bounce_count * 2654435761u;

            let scatter_data = material_scatter(object.material, hit_data, rng_seed);

            // update state
            state.radiance += state.throughput * object.emission_params.xyz;
            state.throughput *= object.material.color;

            if (scatter_data.w < 0.5){ // no bounce
                return packed_color_result(state.radiance);
            }

            let new_photon = Photon3(
                Point3(hit_data.hit_point, CHART_CARTESIAN_WORLD),
                ThreeVector(scatter_data.xyz, TANGENT_CARTESIAN_WORLD)
            );

            state = RayTraceState(
                photon3_to_photon4(new_photon, preferred_chart_for_point(hit_data.hit_point)),
                state.bounce_count + 1u,
                state.radiance,
                state.throughput,
            );
        }
    }

    return packed_color_result(state.radiance);
}

struct Input {
    data: array<Photon4>,
}

struct Output {
    data: array<ColorResult>,
}

struct PackedObjects {
    data: array<PackedObject>,
}

@group(0) @binding(0)
var<storage, read> input_photons: Input;

@group(0) @binding(1)
var<storage, read_write> output_results: Output;

@group(0) @binding(2)
var<storage, read> objects: PackedObjects;

@group(0) @binding(3)
var<storage, read_write> trace_results: TraceResult;

@compute @workgroup_size(64)
fn evolve_rays(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let ray_index = global_id.x;
    let ray_count = arrayLength(&input_photons.data);

    // if (ray_index < 148700) {
    //     output_results.data[ray_index] = packed_color_result(vec3<f32>(1.0, 0.0, 0.0));
    //     return;
    // }

    if (ray_index >= ray_count) {
        return;
    }

    output_results.data[ray_index] = evolve_ray(input_photons.data[ray_index], ray_index);
}