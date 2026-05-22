
// CONSTS

fn sch_g(pos: Point4) -> mat4x4<f32> {
    let r = pos.data.y;
    let theta = pos.data.z;
    let f = 1.0 - R_S / r;

    return mat4x4<f32>(
        vec4<f32>(f, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, -1.0 / f, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, -r * r, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, -r * r * sin(theta) * sin(theta))
    );
}

fn sch_g_inv(pos: Point4) -> mat4x4<f32> {
    let r = pos.data.y;
    let theta = pos.data.z;
    let f = 1.0 - R_S / r;

    return mat4x4<f32>(
        vec4<f32>(1.0 / f, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, R_S / r - 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, -1.0 / (r * r), 0.0),
        vec4<f32>(0.0, 0.0, 0.0, -1.0 / (r * r * sin(theta) * sin(theta)))
    );
}

fn sch_del_g(pos: Point4, chart: u32) -> mat4x4<f32> {
    let r = pos.data.y;
    let theta = pos.data.z;

    if (chart == 0u || chart == 3u) {
        return zero_matrix();
    }

    if (chart == 1u) {
        let f = 1.0 - R_S / r;
        return mat4x4<f32>(
            vec4<f32>(R_S / (r * r), 0.0, 0.0, 0.0),
            vec4<f32>(0.0, R_S / (r * r * f * f), 0.0, 0.0),
            vec4<f32>(0.0, 0.0, -2.0 * r, 0.0),
            vec4<f32>(0.0, 0.0, 0.0, -2.0 * r * sin(theta) * sin(theta))
        );
    }

    if (chart == 2u) {
        return mat4x4<f32>(
            vec4<f32>(0.0, 0.0, 0.0, 0.0),
            vec4<f32>(0.0, 0.0, 0.0, 0.0),
            vec4<f32>(0.0, 0.0, 0.0, 0.0),
            vec4<f32>(0.0, 0.0, 0.0, -2.0 * r * r * cos(theta) * sin(theta))
        );
    }

    return zero_matrix();
}

fn sch_christoffel(pos: Point4, mu: u32, nu: u32, lambda: u32) -> f32 {
    let g_inv = sch_g_inv(pos);
    let d_mu_g = sch_del_g(pos, mu);
    let d_nu_g = sch_del_g(pos, nu);

    var gamma = 0.0;
    for (var alpha: u32 = 0u; alpha < 4u; alpha = alpha + 1u) {
        let d_alpha_g = sch_del_g(pos, alpha);
        gamma = gamma + 0.5 * g_inv[lambda][alpha] * (
            d_mu_g[alpha][nu] + d_nu_g[alpha][mu] - d_alpha_g[mu][nu]
        );
    }

    return gamma;
}

fn sch_null_geodesic_delta_k(x: Point4, k: FourVector) -> FourVector {
    var del_k = four_vector_zero(k.space);

    for (var mu: u32 = 0u; mu < 4u; mu = mu + 1u) {
        for (var alpha: u32 = 0u; alpha < 4u; alpha = alpha + 1u) {
            for (var beta: u32 = 0u; beta < 4u; beta = beta + 1u) {
                let gamma = sch_christoffel(x, alpha, beta, mu);
                del_k.data[mu] = del_k.data[mu] - gamma * k.data[alpha] * k.data[beta];
            }
        }
    }

    return del_k;
}

fn sch_step_along_null_geodesic(photon: Photon4) -> Photon4 {
    let del_k = sch_null_geodesic_delta_k(photon.pos, photon.vel);
    return euler_step(photon.pos, photon.vel, four_vector_as_point4(photon.vel), del_k);
}

fn preferred_chart_for_point(world_pos: vec3<f32>) -> u32 {
    let rel = world_pos - SUBATLAS_CENTER;
    let dist_to_z_axis_sq = rel.x * rel.x + rel.y * rel.y;
    let dist_to_x_axis_sq = rel.y * rel.y + rel.z * rel.z;

    if (dist_to_z_axis_sq < dist_to_x_axis_sq) {
        return SPHERICAL_X;
    }

    return SPHERICAL_Z;
}

fn transition_point_world_to_spherical_z(world_pos: vec3<f32>) -> Point4 {
    let rel = world_pos - SUBATLAS_CENTER;
    let r = length(rel);
    let theta = acos(clamp(rel.z / r, -1.0, 1.0));
    let phi = wrap_tau(atan2(rel.y, rel.x));

    return point4_from_components(0.0, r, theta, phi, SPHERICAL_Z);
}

fn transition_point_world_to_spherical_x(world_pos: vec3<f32>) -> Point4 {
    let rel = world_pos - SUBATLAS_CENTER;
    let r = length(rel);
    let theta = acos(clamp(rel.x / r, -1.0, 1.0));
    let phi = wrap_tau(atan2(rel.z, rel.y));

    return point4_from_components(0.0, r, theta, phi, SPHERICAL_X);
}

fn transition_point_world_to_chart(world_pos: vec3<f32>, chart: u32) -> Point4 {
    if (chart == SPHERICAL_X) {
        return transition_point_world_to_spherical_x(world_pos);
    }

    return transition_point_world_to_spherical_z(world_pos);
}

fn transition_point_chart_to_world(point: Point4) -> vec3<f32> {
    if (point.chart == SPHERICAL_Z) {
        let x = point.data.y * sin(point.data.z) * cos(point.data.w);
        let y = point.data.y * sin(point.data.z) * sin(point.data.w);
        let z = point.data.y * cos(point.data.z);
        return vec3<f32>(x, y, z) + SUBATLAS_CENTER;
    }

    let x = point.data.y * cos(point.data.z);
    let y = point.data.y * sin(point.data.z) * cos(point.data.w);
    let z = point.data.y * sin(point.data.z) * sin(point.data.w);
    return vec3<f32>(x, y, z) + SUBATLAS_CENTER;
}

fn transition_vector_world_to_spherical_z(world_pos: vec3<f32>, world_vel: vec3<f32>) -> FourVector {
    let rel = world_pos - SUBATLAS_CENTER;
    let x = rel.x;
    let y = rel.y;
    let z = rel.z;
    let rho = sqrt(x * x + y * y);
    let r = sqrt(x * x + y * y + z * z);

    let dr_dx = x / r;
    let dr_dy = y / r;
    let dr_dz = z / r;
    let dth_dx = x * z / (r * r * rho);
    let dth_dy = y * z / (r * r * rho);
    let dth_dz = -rho / (r * r);
    let dph_dx = -y / (rho * rho);
    let dph_dy = x / (rho * rho);

    return four_vector_from_components(
        0.0,
        dr_dx * world_vel.x + dr_dy * world_vel.y + dr_dz * world_vel.z,
        dth_dx * world_vel.x + dth_dy * world_vel.y + dth_dz * world_vel.z,
        dph_dx * world_vel.x + dph_dy * world_vel.y,
        SPHERICAL_Z,
    );
}

fn transition_vector_world_to_spherical_x(world_pos: vec3<f32>, world_vel: vec3<f32>) -> FourVector {
    let rel = world_pos - SUBATLAS_CENTER;
    let x = rel.x;
    let y = rel.y;
    let z = rel.z;
    let rho = sqrt(y * y + z * z);
    let r = sqrt(x * x + y * y + z * z);

    let dr_dx = x / r;
    let dr_dy = y / r;
    let dr_dz = z / r;
    let dth_dx = -rho / (r * r);
    let dth_dy = x * y / (r * r * rho);
    let dth_dz = x * z / (r * r * rho);
    let dph_dy = -z / (rho * rho);
    let dph_dz = y / (rho * rho);

    return four_vector_from_components(
        0.0,
        dr_dx * world_vel.x + dr_dy * world_vel.y + dr_dz * world_vel.z,
        dth_dx * world_vel.x + dth_dy * world_vel.y + dth_dz * world_vel.z,
        dph_dy * world_vel.y + dph_dz * world_vel.z,
        SPHERICAL_X,
    );
}

fn transition_vector_world_to_chart(world_pos: vec3<f32>, world_vel: vec3<f32>, chart: u32) -> FourVector {
    if (chart == SPHERICAL_X) {
        return transition_vector_world_to_spherical_x(world_pos, world_vel);
    }

    return transition_vector_world_to_spherical_z(world_pos, world_vel);
}

fn photon_to_world_pos(photon: Photon4) -> vec3<f32> {
    return transition_point_chart_to_world(photon.pos);
}

fn photon_to_world_vel(photon: Photon4) -> vec3<f32> {
    let point = photon.pos;
    let r = point.data.y;
    let theta = point.data.z;
    let phi = point.data.w;
    let v_r = photon.vel.data.y;
    let v_theta = photon.vel.data.z;
    let v_phi = photon.vel.data.w;

    let sin_theta = sin(theta);
    let cos_theta = cos(theta);
    let sin_phi = sin(phi);
    let cos_phi = cos(phi);

    if (point.chart == SPHERICAL_Z) {
        return vec3<f32>(
            sin_theta * cos_phi * v_r + r * cos_theta * cos_phi * v_theta - r * sin_theta * sin_phi * v_phi,
            sin_theta * sin_phi * v_r + r * cos_theta * sin_phi * v_theta + r * sin_theta * cos_phi * v_phi,
            cos_theta * v_r - r * sin_theta * v_theta,
        );
    }

    return vec3<f32>(
        cos_theta * v_r - r * sin_theta * v_theta,
        sin_theta * cos_phi * v_r + r * cos_theta * cos_phi * v_theta - r * sin_theta * sin_phi * v_phi,
        sin_theta * sin_phi * v_r + r * cos_theta * sin_phi * v_theta + r * sin_theta * cos_phi * v_phi,
    );
}

fn world_photon3_to_photon4(world_pos: vec3<f32>, world_vel: vec3<f32>, chart: u32) -> Photon4 {
    let pos = transition_point_world_to_chart(world_pos, chart);
    let vel = transition_vector_world_to_chart(world_pos, world_vel, chart);
    let photon_x = Point4(vec4<f32>(0.0, pos.data.y, pos.data.z, pos.data.w), chart);

    let g = sch_g(photon_x);
    let b = 2.0 * (g[0][1] * vel.data.y + g[0][2] * vel.data.z + g[0][3] * vel.data.w);
    let c =
          g[1][1] * vel.data.y * vel.data.y
        + 2.0 * g[1][2] * vel.data.y * vel.data.z
        + 2.0 * g[1][3] * vel.data.y * vel.data.w
        + g[2][2] * vel.data.z * vel.data.z
        + 2.0 * g[2][3] * vel.data.z * vel.data.w
        + g[3][3] * vel.data.w * vel.data.w;

    let k_0 = quadratic_positive_root(g[0][0], b, c);
    return Photon4(photon_x, FourVector(vec4<f32>(k_0, vel.data.y, vel.data.z, vel.data.w), chart));
}

fn evolve_schwarzschild_ray(photon: Photon4) -> PackedRayResult {
    var current = photon;

    for (var step: u32 = 0u; step < MAX_STEPS; step = step + 1u) {
        current = sch_step_along_null_geodesic(current);

        let world_pos = photon_to_world_pos(current);

        if (current.pos.data.y <= R_S) {
            return PackedRayResult(current, STOP_HORIZON_HIT, vec3<u32>(0u));
        }

        if (length(world_pos - SUBATLAS_CENTER) > SCENE_SIZE) {
            return PackedRayResult(current, STOP_BACKGROUND_REACHED, vec3<u32>(0u));
        }

        let preferred_chart = preferred_chart_for_point(world_pos);
        if (preferred_chart != current.pos.chart) {
            let world_vel = photon_to_world_vel(current);
            current = world_photon3_to_photon4(world_pos, world_vel, preferred_chart);
        }
    }

    return PackedRayResult(current, STOP_MAX_STEPS_REACHED, vec3<u32>(0u));
}

struct InputPhotons {
    data: array<Photon4>,
}

struct OutputResults {
    data: array<PackedRayResult>,
}

@group(0) @binding(0)
var<storage, read> input_photons: InputPhotons;

@group(0) @binding(1)
var<storage, read_write> output_results: OutputResults;

@compute @workgroup_size(64)
fn evolve_schwarzschild_rays(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let ray_index = global_id.x;
    let ray_count = arrayLength(&input_photons.data);

    if (ray_index >= ray_count) {
        return;
    }

    output_results.data[ray_index] = evolve_schwarzschild_ray(input_photons.data[ray_index]);
}