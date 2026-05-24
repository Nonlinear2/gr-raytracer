// common.wgsl

const EULER_STEP_SIZE: f32 = 0.005;
const PI: f32 = 3.141592653589793;
const TAU: f32 = 6.283185307179586;
const MAX_STEPS: u32 = 1000u;

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

struct ColorResult {
    color: vec3<f32>,
    _pad0: f32,
}

fn packed_color_result(color: vec3<f32>) -> ColorResult {
    return ColorResult(color, 0.0);
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
    hit: u32,
}

fn new_ray_trace_state(ray: Photon4) -> RayTraceState {
    return RayTraceState(ray, 0u, VEC3_ZERO, vec3<f32>(1.0, 1.0, 1.0), 0u);
}

// surface.wgsl

struct PackedObject {
    kind: u32,
    material: u32,
    _pad0: u32,
    _pad1: u32,
    data0: vec4<f32>,
    data1: vec4<f32>,
    material_params: vec4<f32>,
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


fn sphere_hit(object: PackedObject, hit_point: Point3) -> bool {
    let center = object.data0.xyz;
    let radius = object.data0.w;
    return length(hit_point.inner - center) <= radius;
}

fn disc_hit(ray_start: vec3<f32>, ray_end: vec3<f32>, object: PackedObject) -> vec4<f32> {
    let center = object.data0.xyz;
    let radius = object.data0.w;

    let normal = normalize(object.data1.xyz);

    let segment = ray_end - ray_start;
    let denom = dot(segment, normal);
    let start_plane_dist = dot(center - ray_start, normal);
    let epsilon = 1e-5;

    if (abs(denom) < epsilon) {
        if (abs(start_plane_dist) <= epsilon && length(ray_start - center) <= radius) {
            return vec4<f32>(ray_start, 1.0);
        }
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let t = start_plane_dist / denom;
    if (t < 0.0 || t > 1.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let hit_point = ray_start + t * segment;
    if (length(hit_point - center) <= radius) {
        return vec4<f32>(hit_point, 1.0);
    }

    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}

fn finish_surface_bounce(
    state: RayTraceState,
    object: PackedObject,
    hit_point: Point3,
    normal: ThreeVector,
    incoming: ThreeVector,
    ray_index: u32,
    step: u32,
    obj_idx: u32,
    new_world_pos: vec3<f32>,
) -> RayTraceState {
    if (state.bounce_count >= MAX_BOUNCES) {
        return RayTraceState(
            state.ray,
            state.bounce_count,
            state.radiance + state.throughput * (object.material_params.xyz + object.emission_params.xyz),
            state.throughput,
            2u,
        );
    }

    let seed_base = ray_index * 73856093u + step * 19349663u + obj_idx * 83492791u + state.bounce_count * 2654435761u;

    var scattered = FourVector(vec4<f32>(0.0, 0.0, 0.0, 0.0), TANGENT_CARTESIAN_WORLD);
    if (object.material == MATERIAL_METAL) {
        scattered = metal_scatter(incoming, normal, hit_point, seed_base, object.material_params.w);
    } else {
        scattered = diffuse_scatter(incoming, normal, hit_point, seed_base);
    }

    if (scattered.inner.w < 0.5) {
        return RayTraceState(
            state.ray,
            state.bounce_count,
            vec3<f32>(235.0 / 255.0, 35.0 / 255.0, 35.0 / 255.0),
            state.throughput,
            2u,
        );
    }

    let scattered_dir = normalize(scattered.inner.xyz);
    let radiance = state.radiance + state.throughput * object.emission_params.xyz;
    let throughput = state.throughput * object.material_params.xyz;
    let bounced_world = Photon3(
        Point3(new_world_pos, CHART_CARTESIAN_WORLD),
        ThreeVector(scattered_dir, TANGENT_CARTESIAN_WORLD),
    );

    return RayTraceState(
        photon3_to_photon4(bounced_world, preferred_chart_for_point(Point3(new_world_pos, CHART_CARTESIAN_WORLD))),
        state.bounce_count + 1u,
        radiance,
        throughput,
        1u,
    );
}

fn resolve_sphere_object(
    state: RayTraceState,
    object: PackedObject,
    hit_point: Point3,
    ray_pos3: Point3,
    ray_index: u32,
    step: u32,
    obj_idx: u32,
) -> RayTraceState {
    let center = object.data0.xyz;
    let radius = object.data0.w;
    let rel = hit_point.inner - center;
    let normal = ThreeVector(normalize(rel), TANGENT_CARTESIAN_WORLD);
    let incoming = ThreeVector(
        normalize(transition_vector(ray_pos3, ThreeVector(state.ray.vel.inner.yzw, state.ray.vel.vector_space), TANGENT_CARTESIAN_WORLD).inner),
        TANGENT_CARTESIAN_WORLD,
    );
    let new_world_pos = center + normal.inner * (radius * 1.000001);

    return finish_surface_bounce(state, object, hit_point, normal, incoming, ray_index, step, obj_idx, new_world_pos);
}

fn resolve_disc_object(
    state: RayTraceState,
    object: PackedObject,
    hit_world: vec3<f32>,
    prev_ray_pos3: Point3,
    prev_ray: Photon4,
    ray_index: u32,
    step: u32,
    obj_idx: u32,
) -> RayTraceState {
    let normal = ThreeVector(normalize(object.data1.xyz), TANGENT_CARTESIAN_WORLD);
    let hit_point = Point3(hit_world, CHART_CARTESIAN_WORLD);
    let incoming = ThreeVector(
        normalize(transition_vector(prev_ray_pos3, ThreeVector(prev_ray.vel.inner.yzw, prev_ray.vel.vector_space), TANGENT_CARTESIAN_WORLD).inner),
        TANGENT_CARTESIAN_WORLD,
    );
    let new_world_pos = hit_world + normal.inner * 0.000001;

    return finish_surface_bounce(state, object, hit_point, normal, incoming, ray_index, step, obj_idx, new_world_pos);
}

fn diffuse_scatter(incoming: ThreeVector, normal: ThreeVector, x: Point3, rng_seed: u32) -> FourVector {
    let rand_dir = random_unit_vector(rng_seed ^ 0xA341316Cu, rng_seed ^ 0xC8013EA4u);
    let dir = normalize(normal.inner + 0.99 * rand_dir);
    return FourVector(vec4<f32>(dir, 1.0), normal.vector_space);
}

fn metal_scatter(incoming: ThreeVector, normal: ThreeVector, x: Point3, rng_seed: u32, fuzz: f32) -> FourVector {
    let rand_dir = random_unit_vector(rng_seed ^ 0xA341316Cu, rng_seed ^ 0xC8013EA4u);

    let f = clamp(fuzz, 0.0, 0.99);
    var scattered_dir = normalize(reflect(incoming.inner, normal.inner) + f * rand_dir);

    if (dot(scattered_dir, normal.inner) <= 0.0) {
        return FourVector(vec4<f32>(-normal.inner, 0.0), normal.vector_space);
    }

    return FourVector(vec4<f32>(scattered_dir, 1.0), normal.vector_space);
}

// euler.wgsl


fn euler_step(x: Point4, k: FourVector, del_x: Point4, del_k: FourVector) -> Photon4 {
    var new_x = new_point4(
        x.inner.x + EULER_STEP_SIZE * del_x.inner.x,
        x.inner.y + EULER_STEP_SIZE * del_x.inner.y,
        x.inner.z + EULER_STEP_SIZE * del_x.inner.z,
        x.inner.w + EULER_STEP_SIZE * del_x.inner.w,
        x.chart
    );

    var new_k = new_four_vector(
        k.inner.x + EULER_STEP_SIZE * del_k.inner.x,
        k.inner.y + EULER_STEP_SIZE * del_k.inner.y,
        k.inner.z + EULER_STEP_SIZE * del_k.inner.z,
        k.inner.w + EULER_STEP_SIZE * del_k.inner.w,
        k.vector_space
    );

    if (x.chart == CHART_SPHERICAL_X || x.chart == CHART_SPHERICAL_Z) {
        var theta = new_x.inner.z;
        var phi = new_x.inner.w;
        var k_theta = new_k.inner.z;

        if (new_x.inner.y < 0.0) {
            new_x.inner.y = 0.0;
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

        new_x.inner.z = clamp(theta, 0.0, PI);
        new_x.inner.w = phi - TAU * floor(phi / TAU); // mod tau
        new_k.inner.z = k_theta;
    }

    return Photon4(new_x, new_k);
}

// schwarzschild.wgsl

// CONSTS
const SUBATLAS_CENTER: vec3<f32> = vec3<f32>(0.0, 0.0, -1.0);  // center is a point in world space
const R_S: f32 = 0.25;
const SCENE_SIZE: f32 = 3.0;


// HasAtlas3 for Schwarzschild4Manifold
// Atlas describing submanifolds of R^4 given by fixing the time coordinate (so this coordinate doesnt get converted).

fn preferred_chart_for_point(point: Point3) -> u32 {
    let point_world = transition_point(point, CHART_CARTESIAN_WORLD);
    let p_rel = point_world.inner - SUBATLAS_CENTER;
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

fn photon3_to_photon4(photon: Photon3, chart: u32) -> Photon4 {
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

fn step_along_null_geodesic(photon: Photon4) -> Photon4 {
    var del_k = four_vector_zero(photon.vel.vector_space);

    for (var mu: u32 = 0u; mu < 4u; mu = mu + 1u) {
        for (var alpha: u32 = 0u; alpha < 4u; alpha = alpha + 1u) {
            for (var beta: u32 = 0u; beta < 4u; beta = beta + 1u) {
                let gamma = christoffel(photon.pos, alpha, beta, mu);
                del_k.inner[mu] = del_k.inner[mu] - gamma * photon.vel.inner[alpha] * photon.vel.inner[beta];
            }
        }
    }

    return euler_step(photon.pos, photon.vel, four_vector_as_point4(photon.vel), del_k);
}

fn evolve_ray(input_ray: Photon4, ray_index: u32) -> ColorResult {
    var state = new_ray_trace_state(input_ray);

    for (var step: u32 = 0u; step < MAX_STEPS; step = step + 1u) {
        let prev_ray = state.ray;
        state.ray = step_along_null_geodesic(state.ray);

        let prev_ray_pos3 = Point3(prev_ray.pos.inner.yzw, prev_ray.pos.chart);
        let ray_pos3 = Point3(state.ray.pos.inner.yzw, state.ray.pos.chart);
        let prev_world_pos = transition_point(prev_ray_pos3, CHART_CARTESIAN_WORLD).inner;
        let world_pos = transition_point(ray_pos3, CHART_CARTESIAN_WORLD).inner;

        if (state.ray.pos.inner.y <= R_S) { // hit singularity
            return packed_color_result(state.radiance);
        }

        if (length(world_pos - SUBATLAS_CENTER) > SCENE_SIZE) { // background reached 
            return packed_color_result(state.radiance + state.throughput * sky_color(world_pos));
        }

        let hit_point = Point3(world_pos, CHART_CARTESIAN_WORLD);
        let preferred_chart = preferred_chart_for_point(hit_point);
        if (preferred_chart != state.ray.pos.chart) {
            let pos3 = Point3(state.ray.pos.inner.yzw, state.ray.pos.chart);
            let vel3 = ThreeVector(state.ray.vel.inner.yzw, state.ray.vel.vector_space);
            state.ray = photon3_to_photon4(Photon3(pos3, vel3), preferred_chart);
        }

        for (var obj_idx: u32 = 0u; obj_idx < arrayLength(&objects.data); obj_idx = obj_idx + 1u) {
            let object = objects.data[obj_idx];
            if (object.kind == OBJECT_SPHERE) {
                if (sphere_hit(object, hit_point)) {
                    state = resolve_sphere_object(state, object, hit_point, ray_pos3, ray_index, step, obj_idx);

                    if (state.hit == 2u) {
                        return packed_color_result(state.radiance);
                    }

                    if (state.hit == 1u) {
                    break; // continue the outer step loop with updated ray
                    }
                }
            } else if (object.kind == OBJECT_DISC) {
                let disc_hit = disc_hit(prev_world_pos, world_pos, object);
                if (disc_hit.w > 0.5) {
                    state = resolve_disc_object(state, object, disc_hit.xyz, prev_ray_pos3, prev_ray, ray_index, step, obj_idx);

                    if (state.hit == 2u) {
                        return packed_color_result(state.radiance);
                    }

                    if (state.hit == 1u) {
                    break;
                    }
                }
            }
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

@compute @workgroup_size(64)
fn evolve_rays(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let ray_index = global_id.x;
    let ray_count = arrayLength(&input_photons.data);

    if (ray_index >= ray_count) {
        return;
    }

    output_results.data[ray_index] = evolve_ray(input_photons.data[ray_index], ray_index);
}